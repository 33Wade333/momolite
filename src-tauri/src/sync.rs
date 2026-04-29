use base64::{engine::general_purpose::STANDARD, Engine as _};
use reqwest::blocking::Client;
use reqwest::{Method, StatusCode};
use rusqlite::{
    params, params_from_iter,
    types::{Value as SqlValue, ValueRef},
    Connection,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::AppHandle;

use crate::db;

const DEFAULT_PROVIDER: &str = "jianguoyun_webdav";
const DEFAULT_BASE_URL: &str = "https://dav.jianguoyun.com/dav/";
const DEFAULT_REMOTE_PATH: &str = "/MomoLite/sync/";
#[cfg(target_os = "windows")]
const KEYRING_SERVICE: &str = "cn.local.momolite";
#[cfg(target_os = "windows")]
const STRONGHOLD_KEY_NAME: &str = "stronghold-vault-key";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncSettings {
    provider: String,
    base_url: String,
    username: String,
    remote_path: String,
    device_id: String,
    auto_sync_enabled: bool,
    last_sync_at: Option<String>,
    sync_status: String,
    last_error: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSyncSettingsRequest {
    provider: String,
    base_url: String,
    username: String,
    remote_path: String,
    auto_sync_enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebDavSecretRequest {
    password: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncActionResult {
    ok: bool,
    message: String,
    remote_snapshot_url: String,
    uploaded_bytes: usize,
    merged_rows: usize,
    synced_at: String,
}

fn now_string() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    seconds.to_string()
}

fn generated_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{prefix}_{nanos}")
}

fn random_secret() -> Result<String, String> {
    let mut bytes = [0_u8; 32];
    getrandom::getrandom(&mut bytes)
        .map_err(|error| format!("failed to create local vault key: {error}"))?;
    Ok(STANDARD.encode(bytes))
}

fn normalize_base_url(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        DEFAULT_BASE_URL.to_string()
    } else {
        trimmed.trim_end_matches('/').to_string()
    }
}

fn normalize_remote_path(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        DEFAULT_REMOTE_PATH.to_string()
    } else {
        let inner = trimmed.trim_matches('/');
        if inner.is_empty() {
            DEFAULT_REMOTE_PATH.to_string()
        } else {
            format!("/{inner}/")
        }
    }
}

fn remote_url(settings: &SyncSettings, suffix: &str) -> String {
    let base = normalize_base_url(&settings.base_url);
    let remote = normalize_remote_path(&settings.remote_path);
    let joined = format!(
        "{}/{}",
        base.trim_end_matches('/'),
        remote.trim_matches('/')
    );
    if suffix.is_empty() {
        joined
    } else {
        format!("{}/{}", joined.trim_end_matches('/'), suffix.trim_start_matches('/'))
    }
}

fn webdav_method(name: &str) -> Result<Method, String> {
    Method::from_bytes(name.as_bytes()).map_err(|error| format!("invalid WebDAV method: {error}"))
}

fn webdav_client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| format!("failed to create WebDAV client: {error}"))
}

fn ensure_settings_row(connection: &Connection) -> Result<SyncSettings, String> {
    let existing = connection
        .query_row(
            "
            SELECT provider, base_url, username, remote_path, device_id,
                auto_sync_enabled, last_sync_at, sync_status, last_error, updated_at
            FROM sync_settings
            WHERE id = 'default'
            ",
            [],
            sync_settings_from_row,
        )
        .ok();

    if let Some(settings) = existing {
        return Ok(settings);
    }

    let updated_at = now_string();
    let device_id = generated_id("device");
    connection
        .execute(
            "
            INSERT INTO sync_settings
                (
                    id, provider, base_url, username, remote_path, device_id,
                    auto_sync_enabled, last_sync_at, sync_status, last_error, updated_at
                )
            VALUES
                ('default', ?1, ?2, '', ?3, ?4, 0, NULL, 'idle', '', ?5)
            ",
            params![
                DEFAULT_PROVIDER,
                DEFAULT_BASE_URL,
                DEFAULT_REMOTE_PATH,
                device_id,
                updated_at
            ],
        )
        .map_err(|error| format!("failed to create sync settings: {error}"))?;

    Ok(SyncSettings {
        provider: DEFAULT_PROVIDER.to_string(),
        base_url: DEFAULT_BASE_URL.to_string(),
        username: String::new(),
        remote_path: DEFAULT_REMOTE_PATH.to_string(),
        device_id,
        auto_sync_enabled: false,
        last_sync_at: None,
        sync_status: "idle".to_string(),
        last_error: String::new(),
        updated_at,
    })
}

fn sync_settings_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SyncSettings> {
    Ok(SyncSettings {
        provider: row.get(0)?,
        base_url: row.get(1)?,
        username: row.get(2)?,
        remote_path: row.get(3)?,
        device_id: row.get(4)?,
        auto_sync_enabled: row.get::<_, i64>(5)? == 1,
        last_sync_at: row.get(6)?,
        sync_status: row.get(7)?,
        last_error: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn update_sync_status(
    connection: &Connection,
    status: &str,
    last_sync_at: Option<&str>,
    last_error: &str,
) -> Result<(), String> {
    let updated_at = now_string();
    connection
        .execute(
            "
            UPDATE sync_settings
            SET sync_status = ?1,
                last_sync_at = COALESCE(?2, last_sync_at),
                last_error = ?3,
                updated_at = ?4
            WHERE id = 'default'
            ",
            params![status, last_sync_at, last_error, updated_at],
        )
        .map_err(|error| format!("failed to update sync status: {error}"))?;
    Ok(())
}

fn ensure_remote_dirs(
    client: &Client,
    settings: &SyncSettings,
    password: &str,
) -> Result<(), String> {
    let base = normalize_base_url(&settings.base_url);
    let mut remote_parts = normalize_remote_path(&settings.remote_path)
        .trim_matches('/')
        .split('/')
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    remote_parts.push("changes".to_string());

    let mkcol = webdav_method("MKCOL")?;
    let mut current = String::new();
    for part in remote_parts {
        current.push('/');
        current.push_str(&part);
        let url = format!("{}/{}", base.trim_end_matches('/'), current.trim_matches('/'));
        let response = client
            .request(mkcol.clone(), &url)
            .basic_auth(&settings.username, Some(password))
            .send()
            .map_err(|error| format!("failed to create remote folder {current}: {error}"))?;
        let status = response.status();
        if !matches!(
            status,
            StatusCode::CREATED | StatusCode::METHOD_NOT_ALLOWED | StatusCode::OK
        ) {
            return Err(format!(
                "failed to prepare remote folder {current}: HTTP {status}"
            ));
        }
    }
    Ok(())
}

fn value_ref_to_json(value: ValueRef<'_>) -> Value {
    match value {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(value) => json!(value),
        ValueRef::Real(value) => json!(value),
        ValueRef::Text(value) => json!(String::from_utf8_lossy(value).to_string()),
        ValueRef::Blob(value) => json!({
            "base64": STANDARD.encode(value),
        }),
    }
}

fn table_as_json(connection: &Connection, table_name: &str) -> Result<Vec<Value>, String> {
    let sql = format!("SELECT * FROM {table_name}");
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| format!("failed to prepare snapshot table {table_name}: {error}"))?;
    let column_names = statement
        .column_names()
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let mut rows = statement
        .query([])
        .map_err(|error| format!("failed to query snapshot table {table_name}: {error}"))?;
    let mut values = Vec::new();

    while let Some(row) = rows
        .next()
        .map_err(|error| format!("failed to read snapshot row from {table_name}: {error}"))?
    {
        let mut object = Map::new();
        for (index, name) in column_names.iter().enumerate() {
            let value = row
                .get_ref(index)
                .map_err(|error| format!("failed to read column {name}: {error}"))?;
            object.insert(name.clone(), value_ref_to_json(value));
        }
        values.push(Value::Object(object));
    }

    Ok(values)
}

fn sync_table_order() -> [&'static str; 18] {
    [
        "course_packs",
        "lessons",
        "sentence_items",
        "review_logs",
        "daily_stats",
        "vocabulary_books",
        "vocabulary_items",
        "vocabulary_book_items",
        "vocabulary_reviews",
        "vocabulary_memory_states",
        "learning_plan_settings",
        "daily_learning_plans",
        "llm_profiles",
        "scene_generation_batches",
        "scene_generation_plans",
        "generated_scenes",
        "scene_lines",
        "scene_vocabulary_links",
    ]
}

fn export_snapshot(connection: &Connection, settings: &SyncSettings) -> Result<Value, String> {
    let mut exported_tables = BTreeMap::new();
    for table in sync_table_order() {
        exported_tables.insert(table.to_string(), Value::Array(table_as_json(connection, table)?));
    }

    Ok(json!({
        "schemaVersion": 1,
        "deviceId": &settings.device_id,
        "exportedAt": now_string(),
        "tables": exported_tables,
    }))
}

fn table_columns_and_primary_key(
    connection: &Connection,
    table_name: &str,
) -> Result<(Vec<String>, String), String> {
    let mut statement = connection
        .prepare(&format!("PRAGMA table_info({table_name})"))
        .map_err(|error| format!("failed to inspect {table_name}: {error}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(1)?, row.get::<_, i64>(5)?))
        })
        .map_err(|error| format!("failed to query table info for {table_name}: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect table info for {table_name}: {error}"))?;
    let columns = rows.iter().map(|(name, _)| name.clone()).collect::<Vec<_>>();
    let primary_key = rows
        .iter()
        .find(|(_, pk)| *pk > 0)
        .map(|(name, _)| name.clone())
        .ok_or_else(|| format!("table {table_name} has no primary key"))?;
    Ok((columns, primary_key))
}

fn json_to_sql_value(value: &Value) -> SqlValue {
    match value {
        Value::Null => SqlValue::Null,
        Value::Bool(value) => SqlValue::Integer(if *value { 1 } else { 0 }),
        Value::Number(value) => {
            if let Some(integer) = value.as_i64() {
                SqlValue::Integer(integer)
            } else if let Some(float) = value.as_f64() {
                SqlValue::Real(float)
            } else {
                SqlValue::Text(value.to_string())
            }
        }
        Value::String(value) => SqlValue::Text(value.clone()),
        Value::Array(_) | Value::Object(_) => SqlValue::Text(value.to_string()),
    }
}

fn json_key(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn existing_updated_at(
    connection: &Connection,
    table_name: &str,
    primary_key: &str,
    primary_value: &str,
) -> Result<Option<String>, String> {
    let sql = format!("SELECT updated_at FROM {table_name} WHERE {primary_key} = ?1");
    match connection.query_row(&sql, params![primary_value], |row| row.get::<_, String>(0)) {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(format!("failed to read local updated_at for {table_name}: {error}")),
    }
}

fn row_exists(
    connection: &Connection,
    table_name: &str,
    primary_key: &str,
    primary_value: &str,
) -> Result<bool, String> {
    let sql = format!("SELECT 1 FROM {table_name} WHERE {primary_key} = ?1 LIMIT 1");
    match connection.query_row(&sql, params![primary_value], |_| Ok(())) {
        Ok(()) => Ok(true),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
        Err(error) => Err(format!("failed to check local row for {table_name}: {error}")),
    }
}

fn should_upsert_remote_row(
    connection: &Connection,
    table_name: &str,
    primary_key: &str,
    primary_value: &str,
    object: &Map<String, Value>,
) -> Result<bool, String> {
    if let Some(remote_updated_at) = object.get("updated_at").and_then(Value::as_str) {
        let local_updated_at =
            existing_updated_at(connection, table_name, primary_key, primary_value)?;
        return Ok(local_updated_at
            .map(|local| remote_updated_at >= local.as_str())
            .unwrap_or(true));
    }

    Ok(!row_exists(
        connection,
        table_name,
        primary_key,
        primary_value,
    )?)
}

fn upsert_remote_row(
    connection: &Connection,
    table_name: &str,
    columns: &[String],
    primary_key: &str,
    object: &Map<String, Value>,
) -> Result<bool, String> {
    let primary_value = object
        .get(primary_key)
        .and_then(json_key)
        .ok_or_else(|| format!("remote {table_name} row is missing primary key {primary_key}"))?;
    if !should_upsert_remote_row(connection, table_name, primary_key, &primary_value, object)? {
        return Ok(false);
    }

    let row_columns = columns
        .iter()
        .filter(|column| object.contains_key(*column))
        .cloned()
        .collect::<Vec<_>>();
    if row_columns.is_empty() {
        return Ok(false);
    }

    let placeholders = (1..=row_columns.len())
        .map(|index| format!("?{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    let update_set = row_columns
        .iter()
        .filter(|column| column.as_str() != primary_key)
        .map(|column| format!("{column} = excluded.{column}"))
        .collect::<Vec<_>>()
        .join(", ");
    let conflict_action = if update_set.is_empty() {
        "DO NOTHING".to_string()
    } else {
        format!("DO UPDATE SET {update_set}")
    };
    let sql = format!(
        "INSERT INTO {table_name} ({}) VALUES ({}) ON CONFLICT({primary_key}) {conflict_action}",
        row_columns.join(", "),
        placeholders
    );
    let values = row_columns
        .iter()
        .map(|column| json_to_sql_value(object.get(column).unwrap_or(&Value::Null)))
        .collect::<Vec<_>>();
    connection
        .execute(&sql, params_from_iter(values))
        .map_err(|error| format!("failed to merge remote {table_name} row: {error}"))?;
    Ok(true)
}

fn merge_remote_snapshot(connection: &Connection, snapshot: &Value) -> Result<usize, String> {
    let tables = snapshot
        .get("tables")
        .and_then(Value::as_object)
        .ok_or_else(|| "remote snapshot has no tables object".to_string())?;
    let mut merged = 0_usize;

    for table_name in sync_table_order() {
        let Some(Value::Array(rows)) = tables.get(table_name) else {
            continue;
        };
        let (columns, primary_key) = table_columns_and_primary_key(connection, table_name)?;
        for row in rows {
            let Some(object) = row.as_object() else {
                continue;
            };
            if upsert_remote_row(connection, table_name, &columns, &primary_key, object)? {
                merged += 1;
            }
        }
    }

    Ok(merged)
}

fn download_remote_snapshot(
    client: &Client,
    settings: &SyncSettings,
    password: &str,
) -> Result<Option<Value>, String> {
    let response = client
        .get(remote_url(settings, "snapshot.json"))
        .basic_auth(&settings.username, Some(password))
        .send()
        .map_err(|error| format!("failed to download remote snapshot: {error}"))?;
    if response.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !response.status().is_success() {
        return Err(format!(
            "failed to download remote snapshot: HTTP {}",
            response.status()
        ));
    }
    let text = response
        .text()
        .map_err(|error| format!("failed to read remote snapshot: {error}"))?;
    serde_json::from_str(&text)
        .map(Some)
        .map_err(|error| format!("remote snapshot is not valid JSON: {error}"))
}

fn require_ready_settings(settings: &SyncSettings, password: &str) -> Result<(), String> {
    if settings.username.trim().is_empty() {
        return Err("WebDAV username is empty".to_string());
    }
    if password.trim().is_empty() {
        return Err("WebDAV app password is empty".to_string());
    }
    if !settings.base_url.starts_with("http://") && !settings.base_url.starts_with("https://") {
        return Err("WebDAV server URL must start with http:// or https://".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn get_secret_vault_key() -> Result<String, String> {
    #[cfg(not(target_os = "windows"))]
    {
        return Err(
            "secure vault key storage is not configured for this platform yet".to_string(),
        );
    }

    #[cfg(target_os = "windows")]
    {
    let entry = keyring::Entry::new(KEYRING_SERVICE, STRONGHOLD_KEY_NAME)
        .map_err(|error| format!("failed to open local credential store: {error}"))?;

    match entry.get_password() {
        Ok(value) if !value.trim().is_empty() => Ok(value),
        _ => {
            let secret = random_secret()?;
            entry
                .set_password(&secret)
                .map_err(|error| format!("failed to save local vault key: {error}"))?;
            Ok(secret)
        }
    }
    }
}

#[tauri::command]
pub fn get_sync_settings(app: AppHandle) -> Result<SyncSettings, String> {
    let connection = db::open_database(&app)?;
    db::create_schema(&connection)?;
    ensure_settings_row(&connection)
}

#[tauri::command]
pub fn save_sync_settings(
    app: AppHandle,
    request: SaveSyncSettingsRequest,
) -> Result<SyncSettings, String> {
    let connection = db::open_database(&app)?;
    db::create_schema(&connection)?;
    let current = ensure_settings_row(&connection)?;
    let updated_at = now_string();

    connection
        .execute(
            "
            INSERT INTO sync_settings
                (
                    id, provider, base_url, username, remote_path, device_id,
                    auto_sync_enabled, last_sync_at, sync_status, last_error, updated_at
                )
            VALUES
                ('default', ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(id) DO UPDATE SET
                provider = excluded.provider,
                base_url = excluded.base_url,
                username = excluded.username,
                remote_path = excluded.remote_path,
                auto_sync_enabled = excluded.auto_sync_enabled,
                updated_at = excluded.updated_at
            ",
            params![
                if request.provider.trim().is_empty() {
                    DEFAULT_PROVIDER
                } else {
                    request.provider.trim()
                },
                normalize_base_url(&request.base_url),
                request.username.trim(),
                normalize_remote_path(&request.remote_path),
                current.device_id,
                if request.auto_sync_enabled { 1 } else { 0 },
                current.last_sync_at,
                current.sync_status,
                current.last_error,
                updated_at,
            ],
        )
        .map_err(|error| format!("failed to save sync settings: {error}"))?;

    ensure_settings_row(&connection)
}

#[tauri::command]
pub fn test_webdav_connection(
    app: AppHandle,
    request: WebDavSecretRequest,
) -> Result<SyncActionResult, String> {
    let connection = db::open_database(&app)?;
    db::create_schema(&connection)?;
    let settings = ensure_settings_row(&connection)?;
    require_ready_settings(&settings, &request.password)?;

    let client = webdav_client()?;
    ensure_remote_dirs(&client, &settings, &request.password)?;

    let propfind = webdav_method("PROPFIND")?;
    let url = remote_url(&settings, "");
    let response = client
        .request(propfind, &url)
        .header("Depth", "0")
        .basic_auth(&settings.username, Some(&request.password))
        .send()
        .map_err(|error| format!("failed to connect to WebDAV: {error}"))?;

    if !response.status().is_success() && response.status() != StatusCode::MULTI_STATUS {
        update_sync_status(
            &connection,
            "failed",
            None,
            &format!("WebDAV test failed: HTTP {}", response.status()),
        )?;
        return Err(format!("WebDAV test failed: HTTP {}", response.status()));
    }

    let synced_at = now_string();
    update_sync_status(&connection, "connected", Some(&synced_at), "")?;
    Ok(SyncActionResult {
        ok: true,
        message: "WebDAV connection ok".to_string(),
        remote_snapshot_url: remote_url(&settings, "snapshot.json"),
        uploaded_bytes: 0,
        merged_rows: 0,
        synced_at,
    })
}

#[tauri::command]
pub fn run_webdav_sync(
    app: AppHandle,
    request: WebDavSecretRequest,
) -> Result<SyncActionResult, String> {
    let connection = db::open_database(&app)?;
    db::create_schema(&connection)?;
    let settings = ensure_settings_row(&connection)?;
    require_ready_settings(&settings, &request.password)?;
    update_sync_status(&connection, "syncing", None, "")?;

    let client = webdav_client()?;
    ensure_remote_dirs(&client, &settings, &request.password)?;

    let merged_rows = match download_remote_snapshot(&client, &settings, &request.password)? {
        Some(snapshot) => merge_remote_snapshot(&connection, &snapshot)?,
        None => 0,
    };
    let snapshot = export_snapshot(&connection, &settings)?;
    let snapshot_bytes = serde_json::to_vec_pretty(&snapshot)
        .map_err(|error| format!("failed to serialize sync snapshot: {error}"))?;
    let snapshot_url = remote_url(&settings, "snapshot.json");
    let device_snapshot_url = remote_url(
        &settings,
        &format!("snapshots-{}.json", settings.device_id),
    );

    for url in [&snapshot_url, &device_snapshot_url] {
        let response = client
            .put(url)
            .basic_auth(&settings.username, Some(&request.password))
            .header("Content-Type", "application/json; charset=utf-8")
            .body(snapshot_bytes.clone())
            .send()
            .map_err(|error| format!("failed to upload sync snapshot: {error}"))?;
        if !response.status().is_success() && response.status() != StatusCode::CREATED {
            let message = format!("snapshot upload failed: HTTP {}", response.status());
            update_sync_status(&connection, "failed", None, &message)?;
            return Err(message);
        }
    }

    let synced_at = now_string();
    let change_line = serde_json::to_string(&json!({
        "id": generated_id("sync_change"),
        "type": "snapshot_uploaded",
        "deviceId": &settings.device_id,
        "syncedAt": synced_at,
        "snapshotBytes": snapshot_bytes.len(),
    }))
    .map_err(|error| format!("failed to serialize sync change line: {error}"))?;
    let changes_url = remote_url(&settings, &format!("changes/{}.jsonl", settings.device_id));
    let response = client
        .put(&changes_url)
        .basic_auth(&settings.username, Some(&request.password))
        .header("Content-Type", "application/x-ndjson; charset=utf-8")
        .body(format!("{change_line}\n"))
        .send()
        .map_err(|error| format!("failed to upload sync changes: {error}"))?;
    if !response.status().is_success() && response.status() != StatusCode::CREATED {
        let message = format!("change log upload failed: HTTP {}", response.status());
        update_sync_status(&connection, "failed", None, &message)?;
        return Err(message);
    }

    update_sync_status(&connection, "synced", Some(&synced_at), "")?;
    Ok(SyncActionResult {
        ok: true,
        message: "Remote snapshot merged and local snapshot uploaded to WebDAV".to_string(),
        remote_snapshot_url: snapshot_url,
        uploaded_bytes: snapshot_bytes.len(),
        merged_rows,
        synced_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_remote_paths_for_webdav() {
        assert_eq!(normalize_remote_path(""), "/MomoLite/sync/");
        assert_eq!(normalize_remote_path("MomoLite/sync"), "/MomoLite/sync/");
        assert_eq!(normalize_remote_path("/MomoLite/sync/"), "/MomoLite/sync/");
    }

    #[test]
    fn merges_remote_snapshot_without_overwriting_newer_local_rows() {
        let connection = Connection::open_in_memory().expect("open memory database");
        db::create_schema(&connection).expect("create schema");
        connection
            .execute(
                "
                INSERT INTO course_packs
                    (id, name, language, daily_new_target, created_at, updated_at)
                VALUES
                    ('course_1', 'Local Course', 'en', 20, '100', '200')
                ",
                [],
            )
            .expect("insert local course");

        let snapshot = json!({
            "schemaVersion": 1,
            "deviceId": "device_remote",
            "tables": {
                "course_packs": [
                    {
                        "id": "course_1",
                        "name": "Older Remote Course",
                        "language": "en",
                        "daily_new_target": 10,
                        "created_at": "100",
                        "updated_at": "150"
                    },
                    {
                        "id": "course_2",
                        "name": "Remote Course",
                        "language": "en",
                        "daily_new_target": 30,
                        "created_at": "300",
                        "updated_at": "300"
                    }
                ]
            }
        });

        let merged = merge_remote_snapshot(&connection, &snapshot).expect("merge snapshot");
        assert_eq!(merged, 1);
        let local_name: String = connection
            .query_row(
                "SELECT name FROM course_packs WHERE id = 'course_1'",
                [],
                |row| row.get(0),
            )
            .expect("read local course");
        let remote_name: String = connection
            .query_row(
                "SELECT name FROM course_packs WHERE id = 'course_2'",
                [],
                |row| row.get(0),
            )
            .expect("read remote course");
        assert_eq!(local_name, "Local Course");
        assert_eq!(remote_name, "Remote Course");
    }
}
