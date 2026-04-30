use reqwest::blocking::Client;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::AppHandle;

use crate::db;

const DEFAULT_PROVIDER: &str = "openai_compatible";
const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1/chat/completions";
const DEFAULT_MODEL: &str = "gpt-4.1-mini";
const DEFAULT_REASONING_EFFORT: &str = "medium";
const DEFAULT_PROMPT_VERSION: &str = "momolite-scene-v1";
const VOLCENGINE_ARK_CHAT_URL: &str = "https://ark.cn-beijing.volces.com/api/v3/chat/completions";
const VOLCENGINE_ENDPOINT_PLACEHOLDER: &str = "";
const GPT2_PROVIDER: &str = "gpt2";
const GPT2_BASE_URL: &str = "https://way.ydata.vip/v1";
const GPT2_MODEL: &str = "gpt-5.4";
const WIRE_CHAT_COMPLETIONS: &str = "chat_completions";
const WIRE_RESPONSES: &str = "responses";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmSettings {
    provider: String,
    base_url: String,
    model: String,
    wire_api: String,
    reasoning_effort: String,
    disable_response_storage: bool,
    prompt_version: String,
    temperature: f64,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmProfile {
    id: String,
    name: String,
    provider: String,
    base_url: String,
    model: String,
    wire_api: String,
    reasoning_effort: String,
    disable_response_storage: bool,
    prompt_version: String,
    temperature: f64,
    is_default: bool,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveLlmSettingsRequest {
    provider: String,
    base_url: String,
    model: String,
    #[serde(default)]
    wire_api: String,
    #[serde(default)]
    reasoning_effort: String,
    #[serde(default)]
    disable_response_storage: bool,
    temperature: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveLlmProfileRequest {
    id: Option<String>,
    name: String,
    provider: String,
    base_url: String,
    model: String,
    #[serde(default)]
    wire_api: String,
    #[serde(default)]
    reasoning_effort: String,
    #[serde(default)]
    disable_response_storage: bool,
    temperature: f64,
    #[serde(default)]
    is_default: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmProfileSecretRequest {
    profile_id: String,
    api_key: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateSceneRequest {
    api_key: String,
    title: String,
    topic: String,
    vocabulary_item_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichVocabularyRequest {
    profile_id: String,
    api_key: String,
    words: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichedVocabularyEntry {
    text: String,
    primary_meaning: String,
    phonetic: String,
    part_of_speech: String,
    example: String,
    example_cn: String,
    roots: String,
    word_family: String,
    synonyms: String,
    antonyms: String,
    memory_hint: String,
    tags: String,
    difficulty: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichVocabularyResponse {
    entries: Vec<EnrichedVocabularyEntry>,
    request_json: String,
    response_json: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanSceneBatchRequest {
    title: String,
    course_pack_id: Option<String>,
    selected_topics: Vec<String>,
    core_word_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateSceneBatchRequest {
    batch_id: String,
    profile_id: String,
    api_key: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddScenesToCourseRequest {
    scene_ids: Vec<String>,
    course_pack_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneGenerationBatch {
    id: String,
    title: String,
    course_pack_id: Option<String>,
    selected_topics: String,
    word_source: String,
    core_word_count: i64,
    support_word_count: i64,
    planned_scene_count: i64,
    status: String,
    coverage_summary: String,
    error_message: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneGenerationPlan {
    id: String,
    batch_id: String,
    sort_order: i64,
    title: String,
    topic: String,
    core_word_ids: String,
    support_word_ids: String,
    core_words_snapshot: String,
    support_words_snapshot: String,
    status: String,
    generated_scene_id: Option<String>,
    error_message: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneBatchPlanResponse {
    batch: SceneGenerationBatch,
    plans: Vec<SceneGenerationPlan>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddScenesToCourseResponse {
    created_lessons: i64,
    created_sentences: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedScene {
    id: String,
    title: String,
    scenario: String,
    prompt_version: String,
    model: String,
    status: String,
    target_words_snapshot: String,
    request_json: String,
    response_json: String,
    error_message: String,
    batch_id: Option<String>,
    planned_course_pack_id: Option<String>,
    lesson_id: Option<String>,
    coverage_json: String,
    is_added_to_course: bool,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneLine {
    id: String,
    scene_id: String,
    sort_order: i64,
    speaker: String,
    english: String,
    chinese: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedSceneDetail {
    scene: GeneratedScene,
    lines: Vec<SceneLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VocabularyTarget {
    id: String,
    text: String,
    normalized_text: String,
    meaning: String,
    difficulty: String,
    weak_score: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScenePayload {
    title: String,
    scenario: String,
    lines: Vec<ScenePayloadLine>,
    #[serde(default)]
    #[serde(rename = "coverageReport")]
    _coverage_report: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScenePayloadLine {
    speaker: String,
    english: String,
    chinese: String,
}

fn now_string() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    seconds.to_string()
}

fn generated_id(prefix: &str, index: usize) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{prefix}_{nanos}_{index}")
}

fn normalize_text(value: &str) -> String {
    value.trim().to_lowercase()
}

fn clamp_temperature(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 2.0)
    } else {
        0.4
    }
}

fn normalize_wire_api(value: &str) -> String {
    match value.trim() {
        WIRE_RESPONSES => WIRE_RESPONSES.to_string(),
        _ => WIRE_CHAT_COMPLETIONS.to_string(),
    }
}

fn normalize_reasoning_effort(value: &str) -> String {
    match value.trim() {
        "minimal" | "low" | "medium" | "high" => value.trim().to_string(),
        _ => DEFAULT_REASONING_EFFORT.to_string(),
    }
}

fn normalize_base_url(value: &str, wire_api: &str) -> String {
    let trimmed = value.trim();
    let wire_api = normalize_wire_api(wire_api);
    if trimmed.is_empty() {
        provider_default_url(DEFAULT_PROVIDER, &wire_api).to_string()
    } else if trimmed.ends_with("/chat/completions") || trimmed.ends_with("/responses") {
        trimmed.to_string()
    } else if wire_api == WIRE_RESPONSES {
        if trimmed.ends_with("/api/v3")
            || trimmed.ends_with("/api/v3/")
            || trimmed.ends_with("/v1")
            || trimmed.ends_with("/v1/")
        {
            format!("{}/responses", trimmed.trim_end_matches('/'))
        } else {
            trimmed.to_string()
        }
    } else if trimmed.ends_with("/api/v3")
        || trimmed.ends_with("/api/v3/")
        || trimmed.ends_with("/v1")
        || trimmed.ends_with("/v1/")
    {
        format!("{}/chat/completions", trimmed.trim_end_matches('/'))
    } else {
        trimmed.to_string()
    }
}

fn provider_default_url(provider: &str, wire_api: &str) -> &'static str {
    match provider {
        GPT2_PROVIDER => GPT2_BASE_URL,
        "volcengine_ark" => VOLCENGINE_ARK_CHAT_URL,
        _ if wire_api == WIRE_RESPONSES => "https://api.openai.com/v1",
        _ => DEFAULT_BASE_URL,
    }
}

fn provider_default_model(provider: &str) -> &'static str {
    match provider {
        GPT2_PROVIDER => GPT2_MODEL,
        "volcengine_ark" => VOLCENGINE_ENDPOINT_PLACEHOLDER,
        _ => DEFAULT_MODEL,
    }
}

fn provider_default_wire_api(provider: &str) -> &'static str {
    match provider {
        GPT2_PROVIDER => WIRE_RESPONSES,
        _ => WIRE_CHAT_COMPLETIONS,
    }
}

fn provider_default_reasoning_effort(provider: &str) -> &'static str {
    match provider {
        GPT2_PROVIDER => "high",
        _ => DEFAULT_REASONING_EFFORT,
    }
}

fn validate_llm_settings(settings: &LlmSettings) -> Result<(), String> {
    if settings.provider == "volcengine_ark" && !settings.model.trim().starts_with("ep-") {
        return Err(
            "火山方舟调用需要在 Model 填写推理接入点 ID，通常以 ep- 开头。请到火山方舟控制台 -> 在线推理 -> 推理接入点，创建/复制接入点 ID；不要直接填写 doubao-seed-2.0-pro。"
                .to_string(),
        );
    }
    if settings.model.trim().is_empty() {
        return Err("LLM Model 不能为空。".to_string());
    }
    if settings.wire_api != WIRE_CHAT_COMPLETIONS && settings.wire_api != WIRE_RESPONSES {
        return Err("wire_api must be chat_completions or responses".to_string());
    }
    Ok(())
}

fn table_has_column(
    connection: &Connection,
    table_name: &str,
    column_name: &str,
) -> Result<bool, String> {
    let mut statement = connection
        .prepare(&format!("PRAGMA table_info({table_name})"))
        .map_err(|error| format!("failed to inspect table {table_name}: {error}"))?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| format!("failed to query columns for {table_name}: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect columns for {table_name}: {error}"))?;
    Ok(columns.iter().any(|name| name == column_name))
}

fn ensure_llm_settings_columns(connection: &Connection) -> Result<(), String> {
    let migrations = [
        (
            "wire_api",
            "ALTER TABLE llm_settings ADD COLUMN wire_api TEXT NOT NULL DEFAULT 'chat_completions'",
        ),
        (
            "reasoning_effort",
            "ALTER TABLE llm_settings ADD COLUMN reasoning_effort TEXT NOT NULL DEFAULT 'medium'",
        ),
        (
            "disable_response_storage",
            "ALTER TABLE llm_settings ADD COLUMN disable_response_storage INTEGER NOT NULL DEFAULT 0 CHECK (disable_response_storage IN (0, 1))",
        ),
    ];

    for (column, sql) in migrations {
        if !table_has_column(connection, "llm_settings", column)? {
            connection
                .execute_batch(sql)
                .map_err(|error| format!("failed to add llm_settings.{column}: {error}"))?;
        }
    }
    Ok(())
}

fn ensure_settings_row(connection: &Connection) -> Result<LlmSettings, String> {
    ensure_llm_settings_columns(connection)?;
    let existing = connection
        .query_row(
            "
            SELECT provider, base_url, model, wire_api, reasoning_effort,
                disable_response_storage, prompt_version, temperature, updated_at
            FROM llm_settings
            WHERE id = 'default'
            ",
            [],
            llm_settings_from_row,
        )
        .ok();

    if let Some(settings) = existing {
        return Ok(settings);
    }

    let updated_at = now_string();
    connection
        .execute(
            "
            INSERT INTO llm_settings
                (
                    id, provider, base_url, model, wire_api, reasoning_effort,
                    disable_response_storage, prompt_version, temperature, updated_at
                )
            VALUES
                ('default', ?1, ?2, ?3, ?4, ?5, 1, ?6, 0.4, ?7)
            ",
            params![
                GPT2_PROVIDER,
                GPT2_BASE_URL,
                GPT2_MODEL,
                WIRE_RESPONSES,
                "high",
                DEFAULT_PROMPT_VERSION,
                updated_at
            ],
        )
        .map_err(|error| format!("failed to create LLM settings: {error}"))?;

    Ok(LlmSettings {
        provider: GPT2_PROVIDER.to_string(),
        base_url: GPT2_BASE_URL.to_string(),
        model: GPT2_MODEL.to_string(),
        wire_api: WIRE_RESPONSES.to_string(),
        reasoning_effort: "high".to_string(),
        disable_response_storage: true,
        prompt_version: DEFAULT_PROMPT_VERSION.to_string(),
        temperature: 0.4,
        updated_at,
    })
}

fn llm_settings_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<LlmSettings> {
    Ok(LlmSettings {
        provider: row.get(0)?,
        base_url: row.get(1)?,
        model: row.get(2)?,
        wire_api: row.get(3)?,
        reasoning_effort: row.get(4)?,
        disable_response_storage: row.get::<_, i64>(5)? == 1,
        prompt_version: row.get(6)?,
        temperature: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn llm_profile_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<LlmProfile> {
    Ok(LlmProfile {
        id: row.get(0)?,
        name: row.get(1)?,
        provider: row.get(2)?,
        base_url: row.get(3)?,
        model: row.get(4)?,
        wire_api: row.get(5)?,
        reasoning_effort: row.get(6)?,
        disable_response_storage: row.get::<_, i64>(7)? == 1,
        prompt_version: row.get(8)?,
        temperature: row.get(9)?,
        is_default: row.get::<_, i64>(10)? == 1,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

fn profile_to_settings(profile: &LlmProfile) -> LlmSettings {
    LlmSettings {
        provider: profile.provider.clone(),
        base_url: profile.base_url.clone(),
        model: profile.model.clone(),
        wire_api: profile.wire_api.clone(),
        reasoning_effort: profile.reasoning_effort.clone(),
        disable_response_storage: profile.disable_response_storage,
        prompt_version: profile.prompt_version.clone(),
        temperature: profile.temperature,
        updated_at: profile.updated_at.clone(),
    }
}

fn list_profiles_on_connection(connection: &Connection) -> Result<Vec<LlmProfile>, String> {
    let mut statement = connection
        .prepare(
            "
            SELECT id, name, provider, base_url, model, wire_api, reasoning_effort,
                disable_response_storage, prompt_version, temperature, is_default,
                created_at, updated_at
            FROM llm_profiles
            ORDER BY is_default DESC, updated_at DESC, name ASC
            ",
        )
        .map_err(|error| format!("failed to prepare LLM profiles: {error}"))?;
    let profiles = statement
        .query_map([], llm_profile_from_row)
        .map_err(|error| format!("failed to query LLM profiles: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect LLM profiles: {error}"))?;
    Ok(profiles)
}

fn get_default_profile(connection: &Connection) -> Result<LlmProfile, String> {
    let profile = connection
        .query_row(
            "
            SELECT id, name, provider, base_url, model, wire_api, reasoning_effort,
                disable_response_storage, prompt_version, temperature, is_default,
                created_at, updated_at
            FROM llm_profiles
            ORDER BY is_default DESC, updated_at DESC
            LIMIT 1
            ",
            [],
            llm_profile_from_row,
        )
        .optional()
        .map_err(|error| format!("failed to load default LLM profile: {error}"))?;

    if let Some(profile) = profile {
        return Ok(profile);
    }

    let settings = ensure_settings_row(connection)?;
    let now = now_string();
    connection
        .execute(
            "
            INSERT INTO llm_profiles
                (
                    id, name, provider, base_url, model, wire_api, reasoning_effort,
                    disable_response_storage, prompt_version, temperature, is_default,
                    created_at, updated_at
                )
            VALUES
                ('default', '我的中转站 GPT-5.4', ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9, ?9)
            ",
            params![
                settings.provider,
                settings.base_url,
                settings.model,
                settings.wire_api,
                settings.reasoning_effort,
                if settings.disable_response_storage { 1 } else { 0 },
                settings.prompt_version,
                settings.temperature,
                now
            ],
        )
        .map_err(|error| format!("failed to seed LLM profile: {error}"))?;
    get_default_profile(connection)
}

fn get_profile_by_id(connection: &Connection, profile_id: &str) -> Result<LlmProfile, String> {
    if profile_id.trim().is_empty() {
        return get_default_profile(connection);
    }
    connection
        .query_row(
            "
            SELECT id, name, provider, base_url, model, wire_api, reasoning_effort,
                disable_response_storage, prompt_version, temperature, is_default,
                created_at, updated_at
            FROM llm_profiles
            WHERE id = ?1
            ",
            params![profile_id],
            llm_profile_from_row,
        )
        .map_err(|error| format!("failed to load LLM profile: {error}"))
}

fn generated_scene_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<GeneratedScene> {
    Ok(GeneratedScene {
        id: row.get(0)?,
        title: row.get(1)?,
        scenario: row.get(2)?,
        prompt_version: row.get(3)?,
        model: row.get(4)?,
        status: row.get(5)?,
        target_words_snapshot: row.get(6)?,
        request_json: row.get(7)?,
        response_json: row.get(8)?,
        error_message: row.get(9)?,
        batch_id: row.get(10)?,
        planned_course_pack_id: row.get(11)?,
        lesson_id: row.get(12)?,
        coverage_json: row.get(13)?,
        is_added_to_course: row.get::<_, i64>(14)? == 1,
        created_at: row.get(15)?,
        updated_at: row.get(16)?,
    })
}

fn load_targets(
    connection: &Connection,
    vocabulary_item_ids: &[String],
) -> Result<Vec<VocabularyTarget>, String> {
    if vocabulary_item_ids.is_empty() {
        let mut statement = connection
            .prepare(
                "
                SELECT id, text, normalized_text, primary_meaning, difficulty, weak_score
                FROM vocabulary_items
                ORDER BY weak_score DESC, COALESCE(next_review_at, created_at) ASC
                LIMIT 8
                ",
            )
            .map_err(|error| format!("failed to prepare default vocabulary targets: {error}"))?;
        return statement
            .query_map([], |row| {
                Ok(VocabularyTarget {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    normalized_text: row.get(2)?,
                    meaning: row.get(3)?,
                    difficulty: row.get(4)?,
                    weak_score: row.get(5)?,
                })
            })
            .map_err(|error| format!("failed to query default vocabulary targets: {error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("failed to collect default vocabulary targets: {error}"));
    }

    let mut targets = Vec::new();
    for id in vocabulary_item_ids {
        let target = connection
            .query_row(
                "
                SELECT id, text, normalized_text, primary_meaning, difficulty, weak_score
                FROM vocabulary_items
                WHERE id = ?1
                ",
                params![id],
                |row| {
                    Ok(VocabularyTarget {
                        id: row.get(0)?,
                        text: row.get(1)?,
                        normalized_text: row.get(2)?,
                        meaning: row.get(3)?,
                        difficulty: row.get(4)?,
                        weak_score: row.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(|error| format!("failed to load vocabulary target {id}: {error}"))?;
        if let Some(target) = target {
            targets.push(target);
        }
    }
    Ok(targets)
}

fn load_smart_core_targets(
    connection: &Connection,
    vocabulary_item_ids: &[String],
) -> Result<Vec<VocabularyTarget>, String> {
    if !vocabulary_item_ids.is_empty() {
        return load_targets(connection, vocabulary_item_ids);
    }

    let mut statement = connection
        .prepare(
            "
            SELECT id, text, normalized_text, primary_meaning, difficulty, weak_score
            FROM vocabulary_items
            WHERE review_count = 0
            ORDER BY created_at ASC
            LIMIT 120
            ",
        )
        .map_err(|error| format!("failed to prepare new word targets: {error}"))?;
    let mut targets = statement
        .query_map([], |row| {
            Ok(VocabularyTarget {
                id: row.get(0)?,
                text: row.get(1)?,
                normalized_text: row.get(2)?,
                meaning: row.get(3)?,
                difficulty: row.get(4)?,
                weak_score: row.get(5)?,
            })
        })
        .map_err(|error| format!("failed to query new word targets: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect new word targets: {error}"))?;

    if targets.is_empty() {
        let mut fallback = connection
            .prepare(
                "
                SELECT id, text, normalized_text, primary_meaning, difficulty, weak_score
                FROM vocabulary_items
                ORDER BY weak_score DESC, COALESCE(next_review_at, created_at) ASC
                LIMIT 120
                ",
            )
            .map_err(|error| format!("failed to prepare fallback word targets: {error}"))?;
        targets = fallback
            .query_map([], |row| {
                Ok(VocabularyTarget {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    normalized_text: row.get(2)?,
                    meaning: row.get(3)?,
                    difficulty: row.get(4)?,
                    weak_score: row.get(5)?,
                })
            })
            .map_err(|error| format!("failed to query fallback word targets: {error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("failed to collect fallback word targets: {error}"))?;
    }
    Ok(targets)
}

fn load_support_targets(
    connection: &Connection,
    core_targets: &[VocabularyTarget],
    limit: usize,
) -> Result<Vec<VocabularyTarget>, String> {
    let core_ids = core_targets
        .iter()
        .map(|target| target.id.as_str())
        .collect::<Vec<_>>();
    let mut statement = connection
        .prepare(
            "
            SELECT id, text, normalized_text, primary_meaning, difficulty, weak_score
            FROM vocabulary_items
            WHERE review_count > 0 OR weak_score > 0
            ORDER BY weak_score DESC, COALESCE(next_review_at, created_at) ASC
            LIMIT 160
            ",
        )
        .map_err(|error| format!("failed to prepare support targets: {error}"))?;
    let targets = statement
        .query_map([], |row| {
            Ok(VocabularyTarget {
                id: row.get(0)?,
                text: row.get(1)?,
                normalized_text: row.get(2)?,
                meaning: row.get(3)?,
                difficulty: row.get(4)?,
                weak_score: row.get(5)?,
            })
        })
        .map_err(|error| format!("failed to query support targets: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect support targets: {error}"))?
        .into_iter()
        .filter(|target| !core_ids.iter().any(|id| *id == target.id))
        .take(limit)
        .collect::<Vec<_>>();
    Ok(targets)
}

fn scene_batch_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SceneGenerationBatch> {
    Ok(SceneGenerationBatch {
        id: row.get(0)?,
        title: row.get(1)?,
        course_pack_id: row.get(2)?,
        selected_topics: row.get(3)?,
        word_source: row.get(4)?,
        core_word_count: row.get(5)?,
        support_word_count: row.get(6)?,
        planned_scene_count: row.get(7)?,
        status: row.get(8)?,
        coverage_summary: row.get(9)?,
        error_message: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

fn scene_plan_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SceneGenerationPlan> {
    Ok(SceneGenerationPlan {
        id: row.get(0)?,
        batch_id: row.get(1)?,
        sort_order: row.get(2)?,
        title: row.get(3)?,
        topic: row.get(4)?,
        core_word_ids: row.get(5)?,
        support_word_ids: row.get(6)?,
        core_words_snapshot: row.get(7)?,
        support_words_snapshot: row.get(8)?,
        status: row.get(9)?,
        generated_scene_id: row.get(10)?,
        error_message: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

fn load_scene_batch_plan(
    connection: &Connection,
    batch_id: &str,
) -> Result<SceneBatchPlanResponse, String> {
    let batch = connection
        .query_row(
            "
            SELECT id, title, course_pack_id, selected_topics, word_source,
                core_word_count, support_word_count, planned_scene_count, status,
                coverage_summary, error_message, created_at, updated_at
            FROM scene_generation_batches
            WHERE id = ?1
            ",
            params![batch_id],
            scene_batch_from_row,
        )
        .map_err(|error| format!("failed to load scene batch: {error}"))?;
    let mut statement = connection
        .prepare(
            "
            SELECT id, batch_id, sort_order, title, topic, core_word_ids,
                support_word_ids, core_words_snapshot, support_words_snapshot,
                status, generated_scene_id, error_message, created_at, updated_at
            FROM scene_generation_plans
            WHERE batch_id = ?1
            ORDER BY sort_order ASC
            ",
        )
        .map_err(|error| format!("failed to prepare scene plans: {error}"))?;
    let plans = statement
        .query_map(params![batch_id], scene_plan_from_row)
        .map_err(|error| format!("failed to query scene plans: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect scene plans: {error}"))?;
    Ok(SceneBatchPlanResponse { batch, plans })
}

fn build_prompt(topic: &str, targets: &[VocabularyTarget]) -> String {
    let target_lines = targets
        .iter()
        .map(|target| {
            format!(
                "- {}: {} | level={} | weak_score={}",
                target.text, target.meaning, target.difficulty, target.weak_score
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"Create one natural daily spoken-English dialogue for MomoLite.

Topic: {topic}

Target vocabulary:
{target_lines}

Rules:
- Use realistic, modern, everyday English.
- Make the dialogue useful for Chinese learners who want to actively use the words.
- Include most target words naturally; do not force awkward sentences.
- Keep it short: 6 to 10 lines.
- Chinese translations must be natural, not word-by-word.
- Return valid JSON only.

JSON schema:
{{
  "title": "short Chinese title",
  "scenario": "one-sentence Chinese scenario",
  "lines": [
    {{
      "speaker": "A",
      "english": "English line",
      "chinese": "Chinese translation"
    }}
  ]
}}"#
    )
}

fn build_scene_lesson_prompt(
    title: &str,
    topic: &str,
    core_targets: &[VocabularyTarget],
    support_targets: &[VocabularyTarget],
    missing_words: &[String],
) -> String {
    let core_lines = core_targets
        .iter()
        .map(|target| {
            format!(
                "- {}: {} | level={} | weak_score={}",
                target.text, target.meaning, target.difficulty, target.weak_score
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let support_lines = support_targets
        .iter()
        .map(|target| format!("- {}: {}", target.text, target.meaning))
        .collect::<Vec<_>>()
        .join("\n");
    let repair_note = if missing_words.is_empty() {
        String::new()
    } else {
        format!(
            "\nRepair requirement: these core words were missing and must be used naturally: {}.\n",
            missing_words.join(", ")
        )
    };

    format!(
        r#"Create one MomoLite scene lesson as a natural native-speaker daily dialogue.

Lesson title hint: {title}
Allowed scene/topic: {topic}

Core target words (must cover 100%, exact word or natural inflection):
{core_lines}

Support/weak old words (use only if natural):
{support_lines}
{repair_note}
Quality rules:
- Write modern, natural spoken English; no textbook example tone.
- Make a real daily situation with useful speaking/listening value.
- 8 to 14 dialogue lines, 2 speakers max unless the scene really needs more.
- Use every core target word at least once.
- Use support words only when they improve the dialogue.
- Chinese translations should be natural.
- Return valid JSON only.

JSON schema:
{{
  "title": "short Chinese lesson title",
  "scenario": "one-sentence Chinese scenario",
  "lines": [
    {{
      "speaker": "A",
      "english": "English dialogue line",
      "chinese": "自然中文翻译"
    }}
  ],
  "coverageReport": {{
    "usedCoreWords": ["word"],
    "missingCoreWords": [],
    "usedSupportWords": ["word"]
  }}
}}"#
    )
}

fn compute_coverage(lines: &[ScenePayloadLine], core_targets: &[VocabularyTarget]) -> Value {
    let joined = lines
        .iter()
        .map(|line| line.english.to_lowercase())
        .collect::<Vec<_>>()
        .join("\n");
    let mut used = Vec::new();
    let mut missing = Vec::new();
    for target in core_targets {
        let normalized = normalize_text(&target.text);
        if !normalized.is_empty() && joined.contains(&normalized) {
            used.push(target.text.clone());
        } else {
            missing.push(target.text.clone());
        }
    }
    json!({
        "usedCoreWords": used,
        "missingCoreWords": missing,
        "coreCoverageRate": if core_targets.is_empty() {
            1.0
        } else {
            (core_targets.len() - missing.len()) as f64 / core_targets.len() as f64
        }
    })
}

fn missing_core_words(coverage: &Value) -> Vec<String> {
    coverage
        .get("missingCoreWords")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn parse_llm_content(response: &Value) -> Result<String, String> {
    let candidates = [
        "/choices/0/message/content",
        "/choices/0/text",
        "/choices/0/delta/content",
        "/data/choices/0/message/content",
        "/result/choices/0/message/content",
        "/output_text",
        "/output/0/content/0/text",
        "/output/0/content/0/content",
        "/content/0/text",
        "/message/content",
        "/data/output_text",
        "/result/output_text",
    ];

    for pointer in candidates {
        if let Some(content) = response.pointer(pointer).and_then(text_from_json_value) {
            if !content.trim().is_empty() {
                return Ok(content.trim().to_string());
            }
        }
    }

    if let Some(content) = response
        .pointer("/choices/0/message/reasoning_content")
        .and_then(Value::as_str)
        .filter(|value| value.trim_start().starts_with('{'))
    {
        return Ok(content.trim().to_string());
    }

    let top_level_keys = response
        .as_object()
        .map(|object| object.keys().cloned().collect::<Vec<_>>().join(", "))
        .unwrap_or_else(|| "non-object response".to_string());
    let preview = serde_json::to_string(response)
        .unwrap_or_default()
        .chars()
        .take(600)
        .collect::<String>();
    Err(format!(
        "LLM response did not contain a supported content field. top_level_keys=[{top_level_keys}], preview={preview}"
    ))
}

fn parse_scene_payload(content: &str) -> Result<ScenePayload, String> {
    let cleaned = content
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    let json_text = extract_json_object_text(cleaned).unwrap_or(cleaned);
    parse_scene_payload_json(json_text)
}

fn text_from_json_value(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Array(values) => {
            let text = values
                .iter()
                .filter_map(text_from_json_value)
                .collect::<Vec<_>>()
                .join("");
            (!text.trim().is_empty()).then_some(text)
        }
        Value::Object(object) => {
            for key in ["text", "content", "value", "output_text"] {
                if let Some(text) = object.get(key).and_then(text_from_json_value) {
                    return Some(text);
                }
            }
            None
        }
        _ => None,
    }
}

fn extract_json_object_text(value: &str) -> Option<&str> {
    let start = value.find('{')?;
    let mut depth = 0_i64;
    let mut in_string = false;
    let mut escaped = false;

    for (offset, character) in value[start..].char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' && in_string {
            escaped = true;
            continue;
        }
        if character == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        if character == '{' {
            depth += 1;
        } else if character == '}' {
            depth -= 1;
            if depth == 0 {
                return Some(&value[start..start + offset + character.len_utf8()]);
            }
        }
    }
    None
}

fn parse_scene_payload_json(value: &str) -> Result<ScenePayload, String> {
    serde_json::from_str(value)
        .map_err(|error| format!("LLM returned invalid scene JSON: {error}"))
}

fn build_enrich_prompt(words: &[String]) -> String {
    let word_lines = words
        .iter()
        .map(|word| format!("- {}", word.trim()))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"Enrich these English vocabulary items for MomoLite, a Chinese learner's focused vocabulary trainer.

Input words:
{word_lines}

Rules:
- Keep one entry for every input word.
- Use accurate Chinese meanings for Chinese learners.
- Prefer modern everyday English and avoid stiff textbook sentences.
- Generate at least 2 natural example sentences for each word, especially speaking/listening scenarios.
- Include common collocations, a short confusion note when useful, and a practical usage tip.
- Include speaking/listening scenario tags such as commute, shopping, work, appointment, family, study, travel, small-talk.
- Keep the JSON schema unchanged. Fold extra learning help into existing fields:
  - Put the strongest example first in "example"; if there is room, append a second short example separated by " / ".
  - Put matching Chinese translations in "exampleCn" using the same order.
  - Put image memory, collocations, confusion note, and usage tip together in "memoryHint".
  - Put scenario tags and topic tags in "tags".
- Return valid JSON only.

JSON schema:
{{
  "entries": [
    {{
      "text": "word or phrase",
      "primaryMeaning": "中文释义",
      "phonetic": "/phonetic/",
      "partOfSpeech": "noun/verb/...",
      "example": "Example 1. / Example 2.",
      "exampleCn": "例句 1 中文。 / 例句 2 中文。",
      "roots": "词根词缀说明；没有则留空",
      "wordFamily": "related forms",
      "synonyms": "synonyms",
      "antonyms": "antonyms",
      "memoryHint": "形象记忆；常见搭配；易混提醒；使用提示",
      "tags": "daily, work, commute, speaking",
      "difficulty": "A1/A2/B1/B2/C1/C2"
    }}
  ]
}}"#
    )
}

fn parse_enriched_entries(content: &str) -> Result<Vec<EnrichedVocabularyEntry>, String> {
    let cleaned = content
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    let json_text = extract_json_object_text(cleaned).unwrap_or(cleaned);
    let value: Value = serde_json::from_str(json_text)
        .map_err(|error| format!("LLM returned invalid vocabulary JSON: {error}"))?;
    if let Some(entries) = value.get("entries") {
        return serde_json::from_value(entries.clone())
            .map_err(|error| format!("LLM vocabulary entries were invalid: {error}"));
    }
    serde_json::from_value(value).map_err(|error| format!("LLM vocabulary entries were invalid: {error}"))
}

fn call_llm(
    settings: &LlmSettings,
    api_key: &str,
    prompt: &str,
) -> Result<(Value, String), String> {
    if api_key.trim().is_empty() {
        return Err("LLM API key is empty".to_string());
    }
    validate_llm_settings(settings)?;
    let client = Client::builder()
        .timeout(Duration::from_secs(90))
        .build()
        .map_err(|error| format!("failed to create LLM HTTP client: {error}"))?;
    let endpoint = normalize_base_url(&settings.base_url, &settings.wire_api);
    let request_json = build_llm_request_json(settings, prompt);
    let response = client
        .post(&endpoint)
        .bearer_auth(api_key.trim())
        .json(&request_json)
        .send()
        .map_err(|error| format!("failed to call LLM at {endpoint}: {error}"))?;
    let status = response.status();
    let response_text = response
        .text()
        .map_err(|error| format!("failed to read LLM response: {error}"))?;
    let response_json: Value = serde_json::from_str(&response_text)
        .unwrap_or_else(|_| json!({ "raw": response_text }));
    if !status.is_success() {
        return Err(format!("LLM request failed: HTTP {status}: {response_json}"));
    }
    let content = parse_llm_content(&response_json)?;
    Ok((response_json, content))
}

fn build_llm_request_json(settings: &LlmSettings, prompt: &str) -> Value {
    if settings.wire_api == WIRE_RESPONSES {
        let request = json!({
            "model": settings.model,
            "input": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "instructions": "You generate structured English learning dialogues. Return JSON only.",
            "reasoning": {
                "effort": settings.reasoning_effort
            },
            "store": !settings.disable_response_storage,
        });
        return request;
    }

    json!({
        "model": settings.model,
        "temperature": settings.temperature,
        "response_format": { "type": "json_object" },
        "messages": [
            {
                "role": "system",
                "content": "You generate structured English learning dialogues. Return JSON only."
            },
            {
                "role": "user",
                "content": prompt
            }
        ]
    })
}

fn save_failed_scene(
    connection: &Connection,
    scene_id: &str,
    error_message: &str,
) -> Result<(), String> {
    let updated_at = now_string();
    connection
        .execute(
            "
            UPDATE generated_scenes
            SET status = 'failed',
                error_message = ?1,
                updated_at = ?2
            WHERE id = ?3
            ",
            params![error_message, updated_at, scene_id],
        )
        .map_err(|error| format!("failed to save failed scene: {error}"))?;
    Ok(())
}

fn save_scene_lines_and_links(
    connection: &Connection,
    scene_id: &str,
    lines: &[ScenePayloadLine],
    targets: &[VocabularyTarget],
) -> Result<(), String> {
    let now = now_string();
    let target_by_text = targets
        .iter()
        .map(|target| (normalize_text(&target.text), target))
        .collect::<HashMap<_, _>>();
    let mut occurrence_order = 0_i64;

    for (index, line) in lines.iter().enumerate() {
        let line_id = generated_id("scene_line", index);
        connection
            .execute(
                "
                INSERT INTO scene_lines
                    (id, scene_id, sort_order, speaker, english, chinese, created_at, updated_at)
                VALUES
                    (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
                ",
                params![
                    &line_id,
                    scene_id,
                    index as i64,
                    line.speaker.trim(),
                    line.english.trim(),
                    line.chinese.trim(),
                    now
                ],
            )
            .map_err(|error| format!("failed to save scene line: {error}"))?;

        let lower = line.english.to_lowercase();
        for (target_text, target) in &target_by_text {
            if target_text.is_empty() {
                continue;
            }
            if let Some(start) = lower.find(target_text) {
                let end = start + target_text.len();
                connection
                    .execute(
                        "
                        INSERT INTO scene_vocabulary_links
                            (
                                id, scene_id, scene_line_id, vocabulary_item_id, matched_text,
                                normalized_target, start_offset, end_offset, occurrence_order,
                                created_at
                            )
                        VALUES
                            (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                        ",
                        params![
                            generated_id("scene_vocab", occurrence_order as usize),
                            scene_id,
                            line_id,
                            &target.id,
                            &line.english[start..end],
                            &target.normalized_text,
                            start as i64,
                            end as i64,
                            occurrence_order,
                            now
                        ],
                    )
                    .map_err(|error| format!("failed to save scene vocabulary link: {error}"))?;
                occurrence_order += 1;
            }
        }
    }
    Ok(())
}

fn load_scene_detail(connection: &Connection, scene_id: &str) -> Result<GeneratedSceneDetail, String> {
    let scene = connection
        .query_row(
            "
            SELECT id, title, scenario, prompt_version, model, status, target_words_snapshot,
                request_json, response_json, error_message, batch_id, planned_course_pack_id,
                lesson_id, coverage_json, is_added_to_course, created_at, updated_at
            FROM generated_scenes
            WHERE id = ?1
            ",
            params![scene_id],
            generated_scene_from_row,
        )
        .map_err(|error| format!("failed to load generated scene: {error}"))?;

    let mut statement = connection
        .prepare(
            "
            SELECT id, scene_id, sort_order, speaker, english, chinese
            FROM scene_lines
            WHERE scene_id = ?1
            ORDER BY sort_order ASC
            ",
        )
        .map_err(|error| format!("failed to prepare scene lines: {error}"))?;
    let lines = statement
        .query_map(params![scene_id], |row| {
            Ok(SceneLine {
                id: row.get(0)?,
                scene_id: row.get(1)?,
                sort_order: row.get(2)?,
                speaker: row.get(3)?,
                english: row.get(4)?,
                chinese: row.get(5)?,
            })
        })
        .map_err(|error| format!("failed to query scene lines: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect scene lines: {error}"))?;

    Ok(GeneratedSceneDetail { scene, lines })
}

#[tauri::command]
pub fn get_llm_settings(app: AppHandle) -> Result<LlmSettings, String> {
    let connection = db::open_database(&app)?;
    db::create_schema(&connection)?;
    get_default_profile(&connection).map(|profile| profile_to_settings(&profile))
}

#[tauri::command]
pub fn save_llm_settings(
    app: AppHandle,
    request: SaveLlmSettingsRequest,
) -> Result<LlmSettings, String> {
    let connection = db::open_database(&app)?;
    db::create_schema(&connection)?;
    let current = ensure_settings_row(&connection)?;
    let updated_at = now_string();
    let provider = if request.provider.trim().is_empty() {
        GPT2_PROVIDER
    } else {
        request.provider.trim()
    };
    let wire_api = if request.wire_api.trim().is_empty() {
        provider_default_wire_api(provider).to_string()
    } else {
        normalize_wire_api(&request.wire_api)
    };
    let base_url = if request.base_url.trim().is_empty() {
        provider_default_url(provider, &wire_api).to_string()
    } else {
        normalize_base_url(&request.base_url, &wire_api)
    };
    let model = if request.model.trim().is_empty() {
        provider_default_model(provider).to_string()
    } else {
        request.model.trim().to_string()
    };
    let reasoning_effort = if request.reasoning_effort.trim().is_empty() {
        provider_default_reasoning_effort(provider).to_string()
    } else {
        normalize_reasoning_effort(&request.reasoning_effort)
    };

    connection
        .execute(
            "
            INSERT INTO llm_settings
                (
                    id, provider, base_url, model, wire_api, reasoning_effort,
                    disable_response_storage, prompt_version, temperature, updated_at
                )
            VALUES
                ('default', ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(id) DO UPDATE SET
                provider = excluded.provider,
                base_url = excluded.base_url,
                model = excluded.model,
                wire_api = excluded.wire_api,
                reasoning_effort = excluded.reasoning_effort,
                disable_response_storage = excluded.disable_response_storage,
                temperature = excluded.temperature,
                updated_at = excluded.updated_at
            ",
            params![
                provider,
                base_url,
                model,
                wire_api,
                reasoning_effort,
                if request.disable_response_storage { 1 } else { 0 },
                &current.prompt_version,
                clamp_temperature(request.temperature),
                updated_at
            ],
        )
        .map_err(|error| format!("failed to save LLM settings: {error}"))?;

    let profile_request = SaveLlmProfileRequest {
        id: Some("default".to_string()),
        name: "我的中转站 GPT-5.4".to_string(),
        provider: provider.to_string(),
        base_url,
        model,
        wire_api,
        reasoning_effort,
        disable_response_storage: request.disable_response_storage,
        temperature: clamp_temperature(request.temperature),
        is_default: true,
    };
    let profile = save_llm_profile_on_connection(&connection, profile_request)?;
    Ok(profile_to_settings(&profile))
}

fn save_llm_profile_on_connection(
    connection: &Connection,
    request: SaveLlmProfileRequest,
) -> Result<LlmProfile, String> {
    let now = now_string();
    let profile_id = request
        .id
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| generated_id("llm_profile", 0));
    let provider = if request.provider.trim().is_empty() {
        GPT2_PROVIDER
    } else {
        request.provider.trim()
    };
    let wire_api = if request.wire_api.trim().is_empty() {
        provider_default_wire_api(provider).to_string()
    } else {
        normalize_wire_api(&request.wire_api)
    };
    let base_url = if request.base_url.trim().is_empty() {
        provider_default_url(provider, &wire_api).to_string()
    } else {
        normalize_base_url(&request.base_url, &wire_api)
    };
    let model = if request.model.trim().is_empty() {
        provider_default_model(provider).to_string()
    } else {
        request.model.trim().to_string()
    };
    let reasoning_effort = if request.reasoning_effort.trim().is_empty() {
        provider_default_reasoning_effort(provider).to_string()
    } else {
        normalize_reasoning_effort(&request.reasoning_effort)
    };
    let profile_name = if request.name.trim().is_empty() {
        "未命名 LLM 配置".to_string()
    } else {
        request.name.trim().to_string()
    };

    let settings = LlmSettings {
        provider: provider.to_string(),
        base_url: base_url.clone(),
        model: model.clone(),
        wire_api: wire_api.clone(),
        reasoning_effort: reasoning_effort.clone(),
        disable_response_storage: request.disable_response_storage,
        prompt_version: DEFAULT_PROMPT_VERSION.to_string(),
        temperature: clamp_temperature(request.temperature),
        updated_at: now.clone(),
    };
    validate_llm_settings(&settings)?;

    if request.is_default {
        connection
            .execute("UPDATE llm_profiles SET is_default = 0", [])
            .map_err(|error| format!("failed to clear default LLM profile: {error}"))?;
    }

    let existing_created_at = connection
        .query_row(
            "SELECT created_at FROM llm_profiles WHERE id = ?1",
            params![&profile_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| format!("failed to inspect LLM profile: {error}"))?;
    let created_at = existing_created_at.unwrap_or_else(|| now.clone());
    connection
        .execute(
            "
            INSERT INTO llm_profiles
                (
                    id, name, provider, base_url, model, wire_api, reasoning_effort,
                    disable_response_storage, prompt_version, temperature, is_default,
                    created_at, updated_at
                )
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                provider = excluded.provider,
                base_url = excluded.base_url,
                model = excluded.model,
                wire_api = excluded.wire_api,
                reasoning_effort = excluded.reasoning_effort,
                disable_response_storage = excluded.disable_response_storage,
                temperature = excluded.temperature,
                is_default = excluded.is_default,
                updated_at = excluded.updated_at
            ",
            params![
                &profile_id,
                &profile_name,
                provider,
                &base_url,
                &model,
                &wire_api,
                &reasoning_effort,
                if request.disable_response_storage { 1 } else { 0 },
                DEFAULT_PROMPT_VERSION,
                clamp_temperature(request.temperature),
                if request.is_default { 1 } else { 0 },
                &created_at,
                &now
            ],
        )
        .map_err(|error| format!("failed to save LLM profile: {error}"))?;
    if !list_profiles_on_connection(connection)?
        .iter()
        .any(|profile| profile.is_default)
    {
        connection
            .execute(
                "UPDATE llm_profiles SET is_default = 1 WHERE id = ?1",
                params![&profile_id],
            )
            .map_err(|error| format!("failed to restore default LLM profile: {error}"))?;
    }
    get_profile_by_id(connection, &profile_id)
}

#[tauri::command]
pub fn list_llm_profiles(app: AppHandle) -> Result<Vec<LlmProfile>, String> {
    let connection = db::open_database(&app)?;
    db::create_schema(&connection)?;
    list_profiles_on_connection(&connection)
}

#[tauri::command]
pub fn save_llm_profile(
    app: AppHandle,
    request: SaveLlmProfileRequest,
) -> Result<LlmProfile, String> {
    let connection = db::open_database(&app)?;
    db::create_schema(&connection)?;
    save_llm_profile_on_connection(&connection, request)
}

#[tauri::command]
pub fn set_default_llm_profile(app: AppHandle, profile_id: String) -> Result<LlmProfile, String> {
    let connection = db::open_database(&app)?;
    db::create_schema(&connection)?;
    get_profile_by_id(&connection, &profile_id)?;
    connection
        .execute("UPDATE llm_profiles SET is_default = 0", [])
        .map_err(|error| format!("failed to clear default profile: {error}"))?;
    let now = now_string();
    connection
        .execute(
            "UPDATE llm_profiles SET is_default = 1, updated_at = ?1 WHERE id = ?2",
            params![&now, &profile_id],
        )
        .map_err(|error| format!("failed to set default profile: {error}"))?;
    get_profile_by_id(&connection, &profile_id)
}

#[tauri::command]
pub fn delete_llm_profile(app: AppHandle, profile_id: String) -> Result<(), String> {
    let connection = db::open_database(&app)?;
    db::create_schema(&connection)?;
    let profiles = list_profiles_on_connection(&connection)?;
    if profiles.len() <= 1 {
        return Err("至少保留一个 LLM 配置。".to_string());
    }
    let was_default = profiles
        .iter()
        .find(|profile| profile.id == profile_id)
        .map(|profile| profile.is_default)
        .unwrap_or(false);
    connection
        .execute("DELETE FROM llm_profiles WHERE id = ?1", params![&profile_id])
        .map_err(|error| format!("failed to delete LLM profile: {error}"))?;
    if was_default {
        if let Some(next_profile) = list_profiles_on_connection(&connection)?.first() {
            connection
                .execute(
                    "UPDATE llm_profiles SET is_default = 1 WHERE id = ?1",
                    params![next_profile.id.clone()],
                )
                .map_err(|error| format!("failed to choose next default profile: {error}"))?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn test_llm_profile(
    app: AppHandle,
    request: LlmProfileSecretRequest,
) -> Result<String, String> {
    let connection = db::open_database(&app)?;
    db::create_schema(&connection)?;
    let profile = get_profile_by_id(&connection, &request.profile_id)?;
    let settings = profile_to_settings(&profile);
    let (_, content) = call_llm(
        &settings,
        &request.api_key,
        r#"Return JSON only: {"ok": true, "message": "MomoLite LLM profile works."}"#,
    )?;
    Ok(content)
}

#[tauri::command]
pub fn enrich_vocabulary_words(
    app: AppHandle,
    request: EnrichVocabularyRequest,
) -> Result<EnrichVocabularyResponse, String> {
    let connection = db::open_database(&app)?;
    db::create_schema(&connection)?;
    let profile = get_profile_by_id(&connection, &request.profile_id)?;
    let settings = profile_to_settings(&profile);
    let words = request
        .words
        .into_iter()
        .map(|word| word.trim().to_string())
        .filter(|word| !word.is_empty())
        .take(80)
        .collect::<Vec<_>>();
    if words.is_empty() {
        return Err("请先输入要补全的单词。".to_string());
    }
    let prompt = build_enrich_prompt(&words);
    let request_snapshot = json!({
        "profileId": profile.id,
        "model": settings.model,
        "promptVersion": "momolite-vocab-enrich-v1",
        "words": words,
        "prompt": prompt,
    });
    let (response_json, content) = call_llm(&settings, &request.api_key, &prompt)?;
    let entries = parse_enriched_entries(&content)?;
    Ok(EnrichVocabularyResponse {
        entries,
        request_json: serde_json::to_string_pretty(&request_snapshot)
            .unwrap_or_else(|_| "{}".to_string()),
        response_json: serde_json::to_string_pretty(&response_json)
            .unwrap_or_else(|_| "{}".to_string()),
    })
}

#[tauri::command]
pub fn list_generated_scenes(app: AppHandle) -> Result<Vec<GeneratedScene>, String> {
    let connection = db::open_database(&app)?;
    db::create_schema(&connection)?;
    let mut statement = connection
        .prepare(
            "
            SELECT id, title, scenario, prompt_version, model, status, target_words_snapshot,
                request_json, response_json, error_message, batch_id, planned_course_pack_id,
                lesson_id, coverage_json, is_added_to_course, created_at, updated_at
            FROM generated_scenes
            ORDER BY updated_at DESC
            ",
        )
        .map_err(|error| format!("failed to prepare generated scenes: {error}"))?;
    let scenes = statement
        .query_map([], generated_scene_from_row)
        .map_err(|error| format!("failed to query generated scenes: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect generated scenes: {error}"))?;
    Ok(scenes)
}

#[tauri::command]
pub fn get_generated_scene(
    app: AppHandle,
    scene_id: String,
) -> Result<GeneratedSceneDetail, String> {
    let connection = db::open_database(&app)?;
    db::create_schema(&connection)?;
    load_scene_detail(&connection, &scene_id)
}

#[tauri::command]
pub fn plan_scene_batch(
    app: AppHandle,
    request: PlanSceneBatchRequest,
) -> Result<SceneBatchPlanResponse, String> {
    let connection = db::open_database(&app)?;
    db::create_schema(&connection)?;
    let core_targets = load_smart_core_targets(&connection, &request.core_word_ids)?;
    if core_targets.is_empty() {
        return Err("还没有可用于规划场景课的单词。".to_string());
    }
    let selected_topics = if request.selected_topics.is_empty() {
        vec![
            "daily work and study".to_string(),
            "commute and errands".to_string(),
            "friends making plans".to_string(),
            "restaurant or cafe".to_string(),
            "planning and decisions".to_string(),
        ]
    } else {
        request
            .selected_topics
            .iter()
            .map(|topic| topic.trim().to_string())
            .filter(|topic| !topic.is_empty())
            .collect::<Vec<_>>()
    };
    let support_targets = load_support_targets(&connection, &core_targets, 60)?;
    let batch_id = generated_id("scene_batch", 0);
    let now = now_string();
    let batch_title = if request.title.trim().is_empty() {
        "AI 场景课批次".to_string()
    } else {
        request.title.trim().to_string()
    };
    let chunks = core_targets
        .chunks(10)
        .map(|chunk| chunk.to_vec())
        .collect::<Vec<_>>();
    let topic_json = serde_json::to_string(&selected_topics)
        .map_err(|error| format!("failed to serialize topics: {error}"))?;
    connection
        .execute(
            "
            INSERT INTO scene_generation_batches
                (
                    id, title, course_pack_id, selected_topics, word_source,
                    core_word_count, support_word_count, planned_scene_count,
                    status, coverage_summary, error_message, created_at, updated_at
                )
            VALUES
                (?1, ?2, ?3, ?4, 'smart', ?5, ?6, ?7, 'planned', '{}', '', ?8, ?8)
            ",
            params![
                batch_id,
                batch_title,
                request.course_pack_id.filter(|value| !value.trim().is_empty()),
                topic_json,
                core_targets.len() as i64,
                support_targets.len() as i64,
                chunks.len() as i64,
                now
            ],
        )
        .map_err(|error| format!("failed to create scene batch: {error}"))?;

    for (index, core_chunk) in chunks.iter().enumerate() {
        let support_start = (index * 5) % support_targets.len().max(1);
        let support_chunk = if support_targets.is_empty() {
            Vec::new()
        } else {
            (0..5)
                .filter_map(|offset| support_targets.get((support_start + offset) % support_targets.len()))
                .cloned()
                .collect::<Vec<_>>()
        };
        let topic = selected_topics
            .get(index % selected_topics.len())
            .cloned()
            .unwrap_or_else(|| "daily life".to_string());
        let plan_id = generated_id("scene_plan", index);
        let plan_title = format!("第 {} 课：{}", index + 1, topic);
        let core_ids = core_chunk
            .iter()
            .map(|target| target.id.clone())
            .collect::<Vec<_>>();
        let support_ids = support_chunk
            .iter()
            .map(|target| target.id.clone())
            .collect::<Vec<_>>();
        connection
            .execute(
                "
                INSERT INTO scene_generation_plans
                    (
                        id, batch_id, sort_order, title, topic, core_word_ids,
                        support_word_ids, core_words_snapshot, support_words_snapshot,
                        status, generated_scene_id, error_message, created_at, updated_at
                    )
                VALUES
                    (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'planned', NULL, '', ?10, ?10)
                ",
                params![
                    plan_id,
                    batch_id,
                    index as i64,
                    plan_title,
                    topic,
                    serde_json::to_string(&core_ids).unwrap_or_else(|_| "[]".to_string()),
                    serde_json::to_string(&support_ids).unwrap_or_else(|_| "[]".to_string()),
                    serde_json::to_string(&core_chunk).unwrap_or_else(|_| "[]".to_string()),
                    serde_json::to_string(&support_chunk).unwrap_or_else(|_| "[]".to_string()),
                    now
                ],
            )
            .map_err(|error| format!("failed to create scene generation plan: {error}"))?;
    }
    load_scene_batch_plan(&connection, &batch_id)
}

#[tauri::command]
pub fn get_scene_batch_plan(
    app: AppHandle,
    batch_id: String,
) -> Result<SceneBatchPlanResponse, String> {
    let connection = db::open_database(&app)?;
    db::create_schema(&connection)?;
    load_scene_batch_plan(&connection, &batch_id)
}

#[tauri::command]
pub fn generate_scene_batch(
    app: AppHandle,
    request: GenerateSceneBatchRequest,
) -> Result<SceneBatchPlanResponse, String> {
    let connection = db::open_database(&app)?;
    db::create_schema(&connection)?;
    let profile = get_profile_by_id(&connection, &request.profile_id)?;
    let settings = profile_to_settings(&profile);
    let current = load_scene_batch_plan(&connection, &request.batch_id)?;
    connection
        .execute(
            "UPDATE scene_generation_batches SET status = 'generating', updated_at = ?1 WHERE id = ?2",
            params![now_string(), request.batch_id],
        )
        .map_err(|error| format!("failed to mark scene batch generating: {error}"))?;

    let mut succeeded = 0_i64;
    let mut failed = 0_i64;
    for plan in current.plans {
        if plan.status == "succeeded" {
            succeeded += 1;
            continue;
        }
        let core_targets: Vec<VocabularyTarget> = serde_json::from_str(&plan.core_words_snapshot)
            .map_err(|error| format!("failed to parse core word snapshot: {error}"))?;
        let support_targets: Vec<VocabularyTarget> = serde_json::from_str(&plan.support_words_snapshot)
            .map_err(|error| format!("failed to parse support word snapshot: {error}"))?;
        let all_targets = core_targets
            .iter()
            .chain(support_targets.iter())
            .cloned()
            .collect::<Vec<_>>();
        let scene_id = generated_id("scene", plan.sort_order as usize);
        let prompt = build_scene_lesson_prompt(
            &plan.title,
            &plan.topic,
            &core_targets,
            &support_targets,
            &[],
        );
        let request_snapshot = json!({
            "batchId": request.batch_id,
            "planId": plan.id,
            "profileId": profile.id,
            "promptVersion": settings.prompt_version,
            "model": settings.model,
            "title": plan.title,
            "topic": plan.topic,
            "coreWords": core_targets,
            "supportWords": support_targets,
            "prompt": prompt,
        });
        let now = now_string();
        connection
            .execute(
                "
                INSERT INTO generated_scenes
                    (
                        id, title, scenario, prompt_version, model, status,
                        target_words_snapshot, request_json, response_json,
                        error_message, batch_id, planned_course_pack_id,
                        coverage_json, is_added_to_course, created_at, updated_at
                    )
                VALUES
                    (?1, ?2, '', ?3, ?4, 'generating', ?5, ?6, '{}', '', ?7, ?8, '{}', 0, ?9, ?9)
                ",
                params![
                    &scene_id,
                    &plan.title,
                    &settings.prompt_version,
                    &settings.model,
                    serde_json::to_string(&all_targets).unwrap_or_else(|_| "[]".to_string()),
                    serde_json::to_string_pretty(&request_snapshot)
                        .unwrap_or_else(|_| "{}".to_string()),
                    &request.batch_id,
                    current.batch.course_pack_id.as_deref(),
                    &now
                ],
            )
            .map_err(|error| format!("failed to create generated scene row: {error}"))?;

        let first_result = call_llm(&settings, &request.api_key, &prompt)
            .and_then(|(response_json, content)| {
                let payload = parse_scene_payload(&content)?;
                Ok((response_json, payload))
            });
        let (response_json, payload) = match first_result {
            Ok((response_json, payload)) => {
                let coverage = compute_coverage(&payload.lines, &core_targets);
                let missing = missing_core_words(&coverage);
                if missing.is_empty() {
                    (response_json, payload)
                } else {
                    let repair_prompt = build_scene_lesson_prompt(
                        &plan.title,
                        &plan.topic,
                        &core_targets,
                        &support_targets,
                        &missing,
                    );
                    call_llm(&settings, &request.api_key, &repair_prompt).and_then(
                        |(repair_response_json, repair_content)| {
                            let repair_payload = parse_scene_payload(&repair_content)?;
                            Ok((repair_response_json, repair_payload))
                        },
                    )?
                }
            }
            Err(error) => {
                save_failed_scene(&connection, &scene_id, &error)?;
                connection
                    .execute(
                        "
                        UPDATE scene_generation_plans
                        SET status = 'failed', error_message = ?1, updated_at = ?2
                        WHERE id = ?3
                        ",
                        params![error, now_string(), plan.id],
                    )
                    .map_err(|error| format!("failed to mark failed plan: {error}"))?;
                failed += 1;
                continue;
            }
        };
        let coverage = compute_coverage(&payload.lines, &core_targets);
        let missing = missing_core_words(&coverage);
        let response_json_text = serde_json::to_string_pretty(&response_json)
            .unwrap_or_else(|_| "{}".to_string());
        let updated_at = now_string();
        if !missing.is_empty() {
            let error = format!("核心词未完全覆盖：{}", missing.join(", "));
            save_failed_scene(&connection, &scene_id, &error)?;
            connection
                .execute(
                    "
                    UPDATE scene_generation_plans
                    SET status = 'failed', error_message = ?1, generated_scene_id = ?2, updated_at = ?3
                    WHERE id = ?4
                    ",
                    params![error, scene_id, updated_at, plan.id],
                )
                .map_err(|error| format!("failed to mark failed coverage plan: {error}"))?;
            failed += 1;
            continue;
        }

        connection
            .execute(
                "
                UPDATE generated_scenes
                SET title = ?1,
                    scenario = ?2,
                    status = 'succeeded',
                    response_json = ?3,
                    coverage_json = ?4,
                    error_message = '',
                    updated_at = ?5
                WHERE id = ?6
                ",
                params![
                    payload.title.trim(),
                    payload.scenario.trim(),
                    response_json_text,
                    serde_json::to_string_pretty(&coverage).unwrap_or_else(|_| "{}".to_string()),
                    updated_at,
                    scene_id
                ],
            )
            .map_err(|error| format!("failed to save generated scene: {error}"))?;
        save_scene_lines_and_links(&connection, &scene_id, &payload.lines, &all_targets)?;
        connection
            .execute(
                "
                UPDATE scene_generation_plans
                SET status = 'succeeded', generated_scene_id = ?1, error_message = '', updated_at = ?2
                WHERE id = ?3
                ",
                params![scene_id, updated_at, plan.id],
            )
            .map_err(|error| format!("failed to mark scene plan succeeded: {error}"))?;
        succeeded += 1;
    }

    let status = if failed == 0 {
        "succeeded"
    } else if succeeded > 0 {
        "partial"
    } else {
        "failed"
    };
    let coverage_summary = json!({
        "succeeded": succeeded,
        "failed": failed,
        "total": succeeded + failed,
    });
    connection
        .execute(
            "
            UPDATE scene_generation_batches
            SET status = ?1, coverage_summary = ?2, updated_at = ?3
            WHERE id = ?4
            ",
            params![
                status,
                serde_json::to_string_pretty(&coverage_summary).unwrap_or_else(|_| "{}".to_string()),
                now_string(),
                request.batch_id
            ],
        )
        .map_err(|error| format!("failed to update scene batch status: {error}"))?;
    load_scene_batch_plan(&connection, &request.batch_id)
}

#[tauri::command]
pub fn add_scenes_to_course(
    app: AppHandle,
    request: AddScenesToCourseRequest,
) -> Result<AddScenesToCourseResponse, String> {
    let connection = db::open_database(&app)?;
    db::create_schema(&connection)?;
    if request.course_pack_id.trim().is_empty() {
        return Err("请选择要加入的课程。".to_string());
    }
    let mut created_lessons = 0_i64;
    let mut created_sentences = 0_i64;
    for scene_id in request.scene_ids {
        let detail = load_scene_detail(&connection, &scene_id)?;
        if detail.scene.status != "succeeded" || detail.scene.is_added_to_course {
            continue;
        }
        let existing_title_count: i64 = connection
            .query_row(
                "
                SELECT COUNT(*)
                FROM lessons
                WHERE course_pack_id = ?1 AND title = ?2
                ",
                params![&request.course_pack_id, &detail.scene.title],
                |row| row.get(0),
            )
            .map_err(|error| format!("failed to check lesson title: {error}"))?;
        let lesson_title = if existing_title_count > 0 {
            format!("{} #{}", detail.scene.title, existing_title_count + 1)
        } else {
            detail.scene.title.clone()
        };
        let sort_order: i64 = connection
            .query_row(
                "
                SELECT COALESCE(MAX(sort_order), -1) + 1
                FROM lessons
                WHERE course_pack_id = ?1
                ",
                params![&request.course_pack_id],
                |row| row.get(0),
            )
            .map_err(|error| format!("failed to compute lesson sort order: {error}"))?;
        let lesson_id = generated_id("lesson", created_lessons as usize);
        let now = now_string();
        connection
            .execute(
                "
                INSERT INTO lessons
                    (id, course_pack_id, title, sort_order, created_at, updated_at)
                VALUES
                    (?1, ?2, ?3, ?4, ?5, ?5)
                ",
                params![
                    &lesson_id,
                    &request.course_pack_id,
                    &lesson_title,
                    sort_order,
                    now
                ],
            )
            .map_err(|error| format!("failed to create lesson from scene: {error}"))?;
        created_lessons += 1;

        for (index, line) in detail.lines.iter().enumerate() {
            let sentence_id = generated_id("sentence", index);
            connection
                .execute(
                    "
                    INSERT INTO sentence_items
                        (id, lesson_id, english, chinese, phonetic, note, created_at, updated_at)
                    VALUES
                        (?1, ?2, ?3, ?4, '', ?5, ?6, ?6)
                    ",
                    params![
                        &sentence_id,
                        &lesson_id,
                        &line.english,
                        &line.chinese,
                        format!("AI 场景课：{}", detail.scene.scenario),
                        now
                    ],
                )
                .map_err(|error| format!("failed to create sentence from scene line: {error}"))?;
            created_sentences += 1;

            let mut link_statement = connection
                .prepare(
                    "
                    SELECT vocabulary_item_id, matched_text
                    FROM scene_vocabulary_links
                    WHERE scene_line_id = ?1
                        AND vocabulary_item_id IS NOT NULL
                    ",
                )
                .map_err(|error| format!("failed to prepare scene vocabulary links: {error}"))?;
            let links = link_statement
                .query_map(params![line.id.clone()], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|error| format!("failed to query scene vocabulary links: {error}"))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| format!("failed to collect scene vocabulary links: {error}"))?;
            for (link_index, (vocabulary_item_id, matched_text)) in links.iter().enumerate() {
                connection
                    .execute(
                        "
                        INSERT OR IGNORE INTO sentence_vocabulary_links
                            (id, sentence_id, vocabulary_item_id, matched_text, created_at)
                        VALUES
                            (?1, ?2, ?3, ?4, ?5)
                        ",
                        params![
                            generated_id("sentence_vocab", link_index),
                            &sentence_id,
                            vocabulary_item_id,
                            matched_text,
                            now
                        ],
                    )
                    .map_err(|error| format!("failed to link sentence vocabulary: {error}"))?;
            }
        }

        connection
            .execute(
                "
                UPDATE generated_scenes
                SET lesson_id = ?1,
                    planned_course_pack_id = ?2,
                    is_added_to_course = 1,
                    updated_at = ?3
                WHERE id = ?4
                ",
                params![&lesson_id, &request.course_pack_id, &now, &scene_id],
            )
            .map_err(|error| format!("failed to mark scene added to course: {error}"))?;
    }
    Ok(AddScenesToCourseResponse {
        created_lessons,
        created_sentences,
    })
}

#[tauri::command]
pub fn generate_scene_from_vocabulary(
    app: AppHandle,
    request: GenerateSceneRequest,
) -> Result<GeneratedSceneDetail, String> {
    let connection = db::open_database(&app)?;
    db::create_schema(&connection)?;
    let settings = profile_to_settings(&get_default_profile(&connection)?);
    let targets = load_targets(&connection, &request.vocabulary_item_ids)?;
    if targets.is_empty() {
        return Err("No vocabulary targets available for scene generation".to_string());
    }

    let scene_id = generated_id("scene", 0);
    let now = now_string();
    let topic = if request.topic.trim().is_empty() {
        "a realistic daily situation".to_string()
    } else {
        request.topic.trim().to_string()
    };
    let prompt = build_prompt(&topic, &targets);
    let target_words_snapshot = serde_json::to_string(&targets)
        .map_err(|error| format!("failed to serialize target word snapshot: {error}"))?;
    let request_json = json!({
        "topic": &topic,
        "title": &request.title,
        "promptVersion": &settings.prompt_version,
        "model": &settings.model,
        "targetWords": &targets,
        "prompt": &prompt,
    });
    let request_json_text = serde_json::to_string_pretty(&request_json)
        .map_err(|error| format!("failed to serialize LLM request snapshot: {error}"))?;

    connection
        .execute(
            "
            INSERT INTO generated_scenes
                (
                    id, title, scenario, prompt_version, model, status, target_words_snapshot,
                    request_json, response_json, error_message, created_at, updated_at
                )
            VALUES
                (?1, ?2, '', ?3, ?4, 'generating', ?5, ?6, '{}', '', ?7, ?7)
            ",
            params![
                scene_id,
                if request.title.trim().is_empty() {
                    "AI 场景对话"
                } else {
                    request.title.trim()
                },
                &settings.prompt_version,
                &settings.model,
                target_words_snapshot,
                request_json_text,
                now
            ],
        )
        .map_err(|error| format!("failed to create generated scene row: {error}"))?;

    let call_result = call_llm(&settings, &request.api_key, &prompt)
        .and_then(|(response_json, content)| {
            let payload = parse_scene_payload(&content)?;
            Ok((response_json, payload))
        });

    let (response_json, payload) = match call_result {
        Ok(value) => value,
        Err(error) => {
            save_failed_scene(&connection, &scene_id, &error)?;
            return Err(error);
        }
    };

    let response_json_text = serde_json::to_string_pretty(&response_json)
        .map_err(|error| format!("failed to serialize LLM response snapshot: {error}"))?;
    let coverage = compute_coverage(&payload.lines, &targets);
    let updated_at = now_string();
    connection
        .execute(
            "
            UPDATE generated_scenes
            SET title = ?1,
                scenario = ?2,
                status = 'succeeded',
                response_json = ?3,
                coverage_json = ?4,
                error_message = '',
                updated_at = ?5
            WHERE id = ?6
            ",
            params![
                payload.title.trim(),
                payload.scenario.trim(),
                response_json_text,
                serde_json::to_string_pretty(&coverage).unwrap_or_else(|_| "{}".to_string()),
                updated_at,
                scene_id
            ],
        )
        .map_err(|error| format!("failed to save generated scene: {error}"))?;

    save_scene_lines_and_links(&connection, &scene_id, &payload.lines, &targets)?;
    load_scene_detail(&connection, &scene_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_scene_payload_from_plain_json() {
        let payload = parse_scene_payload(
            r#"{
                "title": "办公室计划",
                "scenario": "两个人讨论是否继续一个计划。",
                "lines": [
                    {
                        "speaker": "A",
                        "english": "We may have to abandon the plan.",
                        "chinese": "我们可能得放弃这个计划。"
                    }
                ]
            }"#,
        )
        .expect("parse payload");

        assert_eq!(payload.title, "办公室计划");
        assert_eq!(payload.lines.len(), 1);
        assert!(payload.lines[0].english.contains("abandon"));
    }

    #[test]
    fn parses_chat_completion_content_array() {
        let response = json!({
            "choices": [
                {
                    "message": {
                        "content": [
                            { "type": "text", "text": "{\"title\":\"t\",\"scenario\":\"s\",\"lines\":[]}" }
                        ]
                    }
                }
            ]
        });

        let content = parse_llm_content(&response).expect("parse content array");
        assert!(content.contains("\"title\""));
    }

    #[test]
    fn parses_responses_api_output_content() {
        let response = json!({
            "output": [
                {
                    "content": [
                        {
                            "type": "output_text",
                            "text": "{\"title\":\"t\",\"scenario\":\"s\",\"lines\":[]}"
                        }
                    ]
                }
            ]
        });

        let content = parse_llm_content(&response).expect("parse responses output");
        assert!(content.contains("\"scenario\""));
    }

    #[test]
    fn normalizes_provider_endpoint_and_temperature() {
        assert_eq!(
            normalize_base_url("https://ark.cn-beijing.volces.com/api/v3", WIRE_CHAT_COMPLETIONS),
            "https://ark.cn-beijing.volces.com/api/v3/chat/completions"
        );
        assert_eq!(
            normalize_base_url("https://way.ydata.vip/v1", WIRE_RESPONSES),
            "https://way.ydata.vip/v1/responses"
        );
        assert_eq!(clamp_temperature(3.0), 2.0);
        assert_eq!(clamp_temperature(-1.0), 0.0);
    }

    #[test]
    fn builds_responses_request_for_gpt2_gateway() {
        let settings = LlmSettings {
            provider: GPT2_PROVIDER.to_string(),
            base_url: GPT2_BASE_URL.to_string(),
            model: GPT2_MODEL.to_string(),
            wire_api: WIRE_RESPONSES.to_string(),
            reasoning_effort: "high".to_string(),
            disable_response_storage: true,
            prompt_version: DEFAULT_PROMPT_VERSION.to_string(),
            temperature: 0.4,
            updated_at: "100".to_string(),
        };

        let request = build_llm_request_json(&settings, "hello");
        assert_eq!(request["model"], "gpt-5.4");
        assert_eq!(request["reasoning"]["effort"], "high");
        assert_eq!(request["store"], false);
        assert_eq!(request["input"][0]["role"], "user");
    }

    #[test]
    fn validates_volcengine_endpoint_id() {
        let mut settings = LlmSettings {
            provider: "volcengine_ark".to_string(),
            base_url: VOLCENGINE_ARK_CHAT_URL.to_string(),
            model: "doubao-seed-2.0-pro".to_string(),
            wire_api: WIRE_CHAT_COMPLETIONS.to_string(),
            reasoning_effort: DEFAULT_REASONING_EFFORT.to_string(),
            disable_response_storage: false,
            prompt_version: DEFAULT_PROMPT_VERSION.to_string(),
            temperature: 0.4,
            updated_at: "100".to_string(),
        };

        let error = validate_llm_settings(&settings).expect_err("reject base model name");
        assert!(error.contains("ep-"));

        settings.model = "ep-20260428000000-test".to_string();
        validate_llm_settings(&settings).expect("accept endpoint id");
    }

    #[test]
    fn parses_scene_payload_from_json_code_fence() {
        let payload = parse_scene_payload(
            r#"```json
            {
              "title": "学习安排",
              "scenario": "两个同学安排复习。",
              "lines": [
                {
                  "speaker": "B",
                  "english": "Let's review the weak words first.",
                  "chinese": "我们先复习薄弱词吧。"
                }
              ]
            }
            ```"#,
        )
        .expect("parse fenced payload");

        assert_eq!(payload.scenario, "两个同学安排复习。");
        assert_eq!(payload.lines[0].speaker, "B");
    }

    #[test]
    fn saves_scene_lines_and_vocabulary_offsets() {
        let connection = Connection::open_in_memory().expect("open memory database");
        db::create_schema(&connection).expect("create schema");
        connection
            .execute(
                "
                INSERT INTO vocabulary_items
                    (
                        id, text, normalized_text, primary_meaning, phonetic, part_of_speech,
                        example, example_cn, roots, word_family, synonyms, antonyms,
                        memory_hint, tags, difficulty, familiarity, weak_score,
                        review_count, wrong_count, source_book_id, last_reviewed_at,
                        next_review_at, created_at, updated_at
                    )
                VALUES
                    (
                        'word_1', 'abandon', 'abandon', '放弃', '', '',
                        '', '', '', '', '', '', '', '', 'B1', 0, 3,
                        0, 0, NULL, NULL, NULL, '100', '100'
                    )
                ",
                [],
            )
            .expect("insert vocabulary");
        connection
            .execute(
                "
                INSERT INTO generated_scenes
                    (
                        id, title, scenario, prompt_version, model, status,
                        target_words_snapshot, request_json, response_json,
                        error_message, created_at, updated_at
                    )
                VALUES
                    ('scene_1', 'Test', '', 'v1', 'model', 'succeeded', '[]', '{}', '{}', '', '100', '100')
                ",
                [],
            )
            .expect("insert scene");

        let targets = vec![VocabularyTarget {
            id: "word_1".to_string(),
            text: "abandon".to_string(),
            normalized_text: "abandon".to_string(),
            meaning: "放弃".to_string(),
            difficulty: "B1".to_string(),
            weak_score: 3,
        }];
        let lines = vec![ScenePayloadLine {
            speaker: "A".to_string(),
            english: "We cannot abandon the plan yet.".to_string(),
            chinese: "我们还不能放弃这个计划。".to_string(),
        }];

        save_scene_lines_and_links(&connection, "scene_1", &lines, &targets)
            .expect("save lines and links");
        let matched_text: String = connection
            .query_row(
                "SELECT matched_text FROM scene_vocabulary_links WHERE scene_id = 'scene_1'",
                [],
                |row| row.get(0),
            )
            .expect("read matched text");
        assert_eq!(matched_text, "abandon");
    }
}
