use chrono::{DateTime, Duration as ChronoDuration, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

const DB_FILE_NAME: &str = "momolite.sqlite";

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    active_course_pack_id: String,
    course_packs: Vec<CoursePack>,
    sentences: Vec<SentenceItem>,
    reviews: Vec<ReviewLog>,
    stats: HashMap<String, DailyStats>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoursePack {
    id: String,
    name: String,
    language: String,
    daily_new_target: i64,
    created_at: String,
    #[serde(default)]
    updated_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SentenceItem {
    id: String,
    course_pack_id: String,
    lesson_title: String,
    english: String,
    chinese: String,
    phonetic: String,
    note: String,
    status: String,
    favorite: bool,
    show_count: i64,
    review_count: i64,
    error_count: i64,
    next_review_at: Option<String>,
    created_at: String,
    #[serde(default)]
    updated_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewLog {
    id: String,
    sentence_id: String,
    course_pack_id: String,
    user_answer: String,
    is_correct: bool,
    #[serde(default)]
    wrong_indexes: Vec<usize>,
    rating: String,
    reviewed_at: String,
    interval_minutes: i64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyStats {
    new_count: i64,
    review_count: i64,
    study_minutes: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyBook {
    id: String,
    name: String,
    source: String,
    imported_at: String,
    note: String,
    item_count: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyItem {
    id: String,
    text: String,
    normalized_text: String,
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
    familiarity: i64,
    weak_score: i64,
    review_count: i64,
    wrong_count: i64,
    last_reviewed_at: Option<String>,
    next_review_at: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyBookDetail {
    book: VocabularyBook,
    items: Vec<VocabularyItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportVocabularyBookRequest {
    name: String,
    source: String,
    raw_text: String,
    note: String,
    imported_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportVocabularyBookResult {
    book: VocabularyBook,
    items: Vec<VocabularyItem>,
    imported_count: i64,
    reused_count: i64,
    skipped_count: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyReviewSubmission {
    vocabulary_item_id: String,
    mode: String,
    rating: String,
    reviewed_at: String,
    next_review_at: String,
}

#[derive(Debug, Clone)]
struct VocabularyMemoryState {
    id: String,
    vocabulary_item_id: String,
    skill_type: String,
    state: String,
    stability: f64,
    difficulty: f64,
    retrievability: f64,
    interval_days: f64,
    due_at: Option<String>,
    last_reviewed_at: Option<String>,
    lapse_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningPlanSettings {
    intensity: String,
    desired_retention: f64,
    daily_new_target: i64,
    daily_review_limit: i64,
    scene_lessons_target: i64,
    recovery_days: i64,
    updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyLearningPlan {
    id: String,
    plan_date: String,
    intensity: String,
    new_word_target: i64,
    review_limit: i64,
    scene_lesson_target: i64,
    due_count: i64,
    weak_count: i64,
    backlog_count: i64,
    plan_json: String,
    explanation: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveLearningPlanSettingsRequest {
    intensity: String,
}

#[derive(Debug, Clone, Serialize)]
struct ParsedVocabularyEntry {
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
    source_line: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCoursePack {
    id: String,
    name: String,
    language: String,
    daily_new_target: i64,
    created_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewLesson {
    id: String,
    course_pack_id: String,
    title: String,
    sort_order: i64,
    created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSentenceItem {
    id: String,
    lesson_id: String,
    english: String,
    chinese: String,
    phonetic: String,
    note: String,
    created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewSubmission {
    id: String,
    sentence_id: String,
    user_answer: String,
    is_correct: bool,
    #[serde(default)]
    wrong_indexes: Vec<usize>,
    rating: String,
    reviewed_at: String,
    interval_minutes: i64,
    next_review_at: Option<String>,
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

fn add_column_if_missing(
    connection: &Connection,
    table_name: &str,
    column_name: &str,
    ddl: &str,
) -> Result<(), String> {
    if !table_has_column(connection, table_name, column_name)? {
        connection
            .execute(ddl, [])
            .map_err(|error| format!("failed to add {table_name}.{column_name}: {error}"))?;
    }
    Ok(())
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

fn clamp_score(value: i64, min: i64, max: i64) -> i64 {
    value.max(min).min(max)
}

fn normalize_vocabulary_text(value: &str) -> String {
    let cleaned = value
        .trim()
        .trim_matches(|character: char| {
            !character.is_alphanumeric() && character != '\'' && character != '-'
        })
        .to_lowercase();

    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn seed_default_llm_profile(connection: &Connection) -> Result<(), String> {
    let existing_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM llm_profiles", [], |row| row.get(0))
        .map_err(|error| format!("failed to count LLM profiles: {error}"))?;
    if existing_count > 0 {
        return Ok(());
    }

    let now = now_string();
    let copied = connection
        .execute(
            "
            INSERT INTO llm_profiles
                (
                    id, name, provider, base_url, model, wire_api, reasoning_effort,
                    disable_response_storage, prompt_version, temperature, is_default,
                    created_at, updated_at
                )
            SELECT
                'default',
                '我的中转站 GPT-5.4',
                provider,
                base_url,
                model,
                wire_api,
                reasoning_effort,
                disable_response_storage,
                prompt_version,
                temperature,
                1,
                ?1,
                updated_at
            FROM llm_settings
            WHERE id = 'default'
            ",
            params![now],
        )
        .map_err(|error| format!("failed to migrate default LLM profile: {error}"))?;

    if copied == 0 {
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
                    (
                        'default', '我的中转站 GPT-5.4', 'gpt2',
                        'https://way.ydata.vip/v1', 'gpt-5.4', 'responses', 'high',
                        1, 'momolite-scene-v1', 0.4, 1, ?1, ?1
                    )
                ",
                params![now],
            )
            .map_err(|error| format!("failed to seed default LLM profile: {error}"))?;
    }
    Ok(())
}

fn seed_learning_plan_settings(connection: &Connection) -> Result<(), String> {
    let now = now_string();
    connection
        .execute(
            "
            INSERT OR IGNORE INTO learning_plan_settings
                (
                    id, intensity, desired_retention, daily_new_target,
                    daily_review_limit, scene_lessons_target, recovery_days, updated_at
                )
            VALUES
                ('default', 'standard', 0.90, 25, 160, 3, 3, ?1)
            ",
            params![now],
        )
        .map_err(|error| format!("failed to seed learning plan settings: {error}"))?;
    Ok(())
}

fn parse_review_time(value: &str) -> DateTime<Utc> {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(value) {
        return parsed.with_timezone(&Utc);
    }
    if let Ok(seconds) = value.parse::<i64>() {
        if let Some(parsed) = DateTime::<Utc>::from_timestamp(seconds, 0) {
            return parsed;
        }
    }
    Utc::now()
}

fn add_days_iso(base: &str, days: f64) -> String {
    let minutes = (days.max(0.0) * 1440.0).round() as i64;
    (parse_review_time(base) + ChronoDuration::minutes(minutes)).to_rfc3339()
}

fn days_between(start: &str, end: &str) -> f64 {
    let start = parse_review_time(start);
    let end = parse_review_time(end);
    let seconds = (end - start).num_seconds().max(0) as f64;
    seconds / 86_400.0
}

fn memory_retrievability(stability: f64, elapsed_days: f64) -> f64 {
    if stability <= 0.0 {
        return 0.0;
    }
    0.9_f64
        .powf(elapsed_days.max(0.0) / stability.max(0.01))
        .clamp(0.0, 1.0)
}

fn initial_memory_state(vocabulary_item_id: &str, skill_type: &str) -> VocabularyMemoryState {
    let now = now_string();
    VocabularyMemoryState {
        id: generated_id("vocab_memory", 0),
        vocabulary_item_id: vocabulary_item_id.to_string(),
        skill_type: skill_type.to_string(),
        state: "new".to_string(),
        stability: 0.0,
        difficulty: 5.0,
        retrievability: 0.0,
        interval_days: 0.0,
        due_at: None,
        last_reviewed_at: None,
        lapse_count: 0,
    }
    .with_created_at_fallback(now)
}

impl VocabularyMemoryState {
    fn with_created_at_fallback(self, _now: String) -> Self {
        self
    }
}

fn memory_state_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<VocabularyMemoryState> {
    Ok(VocabularyMemoryState {
        id: row.get(0)?,
        vocabulary_item_id: row.get(1)?,
        skill_type: row.get(2)?,
        state: row.get(3)?,
        stability: row.get(4)?,
        difficulty: row.get(5)?,
        retrievability: row.get(6)?,
        interval_days: row.get(7)?,
        due_at: row.get(8)?,
        last_reviewed_at: row.get(9)?,
        lapse_count: row.get(10)?,
    })
}

fn load_memory_state(
    connection: &Connection,
    vocabulary_item_id: &str,
    skill_type: &str,
) -> Result<VocabularyMemoryState, String> {
    let existing = connection
        .query_row(
            "
            SELECT id, vocabulary_item_id, skill_type, state, stability, difficulty,
                retrievability, interval_days, due_at, last_reviewed_at, lapse_count
            FROM vocabulary_memory_states
            WHERE vocabulary_item_id = ?1 AND skill_type = ?2
            ",
            params![vocabulary_item_id, skill_type],
            memory_state_from_row,
        )
        .optional()
        .map_err(|error| format!("failed to load memory state: {error}"))?;
    Ok(existing.unwrap_or_else(|| initial_memory_state(vocabulary_item_id, skill_type)))
}

fn apply_memory_rating(
    current: &VocabularyMemoryState,
    rating: &str,
    reviewed_at: &str,
) -> Result<VocabularyMemoryState, String> {
    let elapsed_days = current
        .last_reviewed_at
        .as_ref()
        .map(|last| days_between(last, reviewed_at))
        .unwrap_or(0.0);
    let retrievability_before = memory_retrievability(current.stability, elapsed_days);
    let mut next = current.clone();
    next.retrievability = retrievability_before;

    let success = matches!(rating, "good" | "easy");
    let base_stability = if current.stability <= 0.0 {
        match rating {
            "again" => 10.0 / 1440.0,
            "hard" => 1.0,
            "good" => 3.0,
            "easy" => 7.0,
            _ => return Err("invalid vocabulary rating".to_string()),
        }
    } else {
        current.stability
    };

    match rating {
        "again" => {
            next.state = "relearning".to_string();
            next.stability = (base_stability * 0.35).max(10.0 / 1440.0);
            next.difficulty = (current.difficulty + 0.9).min(10.0);
            next.interval_days = 10.0 / 1440.0;
            next.lapse_count += 1;
        }
        "hard" => {
            next.state = "learning".to_string();
            next.stability = base_stability.max(1.0) * 0.85;
            next.difficulty = (current.difficulty + 0.45).min(10.0);
            next.interval_days = 1.0;
            next.lapse_count += 1;
        }
        "good" => {
            next.state = "review".to_string();
            let growth = 1.0 + (11.0 - current.difficulty).max(1.0) * 0.18;
            next.stability = base_stability.max(2.0) * growth;
            next.difficulty = (current.difficulty - 0.15).max(1.0);
            next.interval_days = next.stability.max(2.0);
        }
        "easy" => {
            next.state = if current.review_count_hint() >= 2 {
                "mastered".to_string()
            } else {
                "review".to_string()
            };
            let growth = 1.45 + (11.0 - current.difficulty).max(1.0) * 0.22;
            next.stability = base_stability.max(4.0) * growth;
            next.difficulty = (current.difficulty - 0.35).max(1.0);
            next.interval_days = next.stability.max(5.0);
        }
        _ => return Err("invalid vocabulary rating".to_string()),
    }

    if !success && next.interval_days > 1.0 {
        next.interval_days = 1.0;
    }
    next.retrievability = if success { 1.0 } else { 0.35 };
    next.due_at = Some(add_days_iso(reviewed_at, next.interval_days));
    next.last_reviewed_at = Some(reviewed_at.to_string());
    Ok(next)
}

impl VocabularyMemoryState {
    fn review_count_hint(&self) -> i64 {
        if self.last_reviewed_at.is_some() {
            1 + self.lapse_count
        } else {
            0
        }
    }
}

fn save_memory_state(connection: &Connection, state: &VocabularyMemoryState) -> Result<(), String> {
    let now = now_string();
    connection
        .execute(
            "
            INSERT INTO vocabulary_memory_states
                (
                    id, vocabulary_item_id, skill_type, state, stability, difficulty,
                    retrievability, interval_days, due_at, last_reviewed_at,
                    lapse_count, created_at, updated_at
                )
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12)
            ON CONFLICT(vocabulary_item_id, skill_type) DO UPDATE SET
                state = excluded.state,
                stability = excluded.stability,
                difficulty = excluded.difficulty,
                retrievability = excluded.retrievability,
                interval_days = excluded.interval_days,
                due_at = excluded.due_at,
                last_reviewed_at = excluded.last_reviewed_at,
                lapse_count = excluded.lapse_count,
                updated_at = excluded.updated_at
            ",
            params![
                state.id,
                state.vocabulary_item_id,
                state.skill_type,
                state.state,
                state.stability,
                state.difficulty,
                state.retrievability,
                state.interval_days,
                state.due_at,
                state.last_reviewed_at,
                state.lapse_count,
                now
            ],
        )
        .map_err(|error| format!("failed to save memory state: {error}"))?;
    Ok(())
}

fn update_vocabulary_memory(
    connection: &Connection,
    vocabulary_item_id: &str,
    skill_type: &str,
    mode: &str,
    rating: &str,
    reviewed_at: &str,
) -> Result<(VocabularyMemoryState, VocabularyMemoryState), String> {
    let before = load_memory_state(connection, vocabulary_item_id, skill_type)?;
    let after = apply_memory_rating(&before, rating, reviewed_at)?;
    save_memory_state(connection, &after)?;
    connection
        .execute(
            "
            INSERT INTO vocabulary_reviews
                (
                    id, vocabulary_item_id, mode, rating, reviewed_at, next_review_at,
                    skill_type, stability_before, stability_after, difficulty_before,
                    difficulty_after, retrievability_before, retrievability_after
                )
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ",
            params![
                generated_id("vocab_review", 0),
                vocabulary_item_id,
                mode,
                rating,
                reviewed_at,
                after.due_at.clone().unwrap_or_else(|| reviewed_at.to_string()),
                skill_type,
                before.stability,
                after.stability,
                before.difficulty,
                after.difficulty,
                before.retrievability,
                after.retrievability
            ],
        )
        .map_err(|error| format!("failed to insert vocabulary review: {error}"))?;
    Ok((before, after))
}

fn parse_field_line(line: &str) -> Option<(String, String)> {
    let trimmed = line
        .trim()
        .strip_prefix('-')
        .or_else(|| line.trim().strip_prefix('*'))?
        .trim();
    let (key, value) = trimmed
        .split_once('：')
        .or_else(|| trimmed.split_once(':'))?;
    let key = key.trim().to_string();
    let value = value.trim().to_string();
    if key.is_empty() || value.is_empty() {
        return None;
    }
    Some((key, value))
}

fn apply_vocabulary_field(entry: &mut ParsedVocabularyEntry, key: &str, value: String) {
    match key.trim() {
        "中文释义" | "释义" | "中文" | "意思" => entry.primary_meaning = value,
        "音标" | "发音" => entry.phonetic = value,
        "词性" => entry.part_of_speech = value,
        "例句" | "英文例句" => entry.example = value,
        "例句中文" | "中文例句" | "例句翻译" => entry.example_cn = value,
        "词根词缀" | "词根" | "词缀" => entry.roots = value,
        "同根词" | "词族" | "派生词" => entry.word_family = value,
        "近义词" | "同义词" => entry.synonyms = value,
        "反义词" => entry.antonyms = value,
        "形象记忆" | "记忆法" | "记忆提示" => entry.memory_hint = value,
        "场景标签" | "标签" | "场景" => entry.tags = value,
        "难度" => entry.difficulty = value,
        _ => {}
    }
}

fn empty_vocabulary_entry(text: String, source_line: i64) -> ParsedVocabularyEntry {
    ParsedVocabularyEntry {
        text,
        primary_meaning: String::new(),
        phonetic: String::new(),
        part_of_speech: String::new(),
        example: String::new(),
        example_cn: String::new(),
        roots: String::new(),
        word_family: String::new(),
        synonyms: String::new(),
        antonyms: String::new(),
        memory_hint: String::new(),
        tags: String::new(),
        difficulty: String::new(),
        source_line,
    }
}

fn parse_vocabulary_book_markdown(
    raw_text: &str,
) -> (Option<String>, Vec<ParsedVocabularyEntry>, i64) {
    let mut title = None;
    let mut entries = Vec::new();
    let mut skipped = 0;
    let mut current: Option<ParsedVocabularyEntry> = None;

    for (index, line) in raw_text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("# ") && title.is_none() {
            title = Some(trimmed.trim_start_matches("# ").trim().to_string());
            continue;
        }

        if trimmed.starts_with("## ") {
            if let Some(entry) = current.take() {
                if entry.text.trim().is_empty() || entry.primary_meaning.trim().is_empty() {
                    skipped += 1;
                } else {
                    entries.push(entry);
                }
            }
            current = Some(empty_vocabulary_entry(
                trimmed.trim_start_matches("## ").trim().to_string(),
                index as i64 + 1,
            ));
            continue;
        }

        if let Some((key, value)) = parse_field_line(trimmed) {
            if let Some(entry) = current.as_mut() {
                apply_vocabulary_field(entry, &key, value);
            }
        }
    }

    if let Some(entry) = current.take() {
        if entry.text.trim().is_empty() || entry.primary_meaning.trim().is_empty() {
            skipped += 1;
        } else {
            entries.push(entry);
        }
    }

    (title, entries, skipped)
}

pub(crate) fn database_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("failed to resolve app data dir: {error}"))?;

    fs::create_dir_all(&app_dir)
        .map_err(|error| format!("failed to create app data dir: {error}"))?;

    Ok(app_dir.join(DB_FILE_NAME))
}

pub(crate) fn open_database(app: &AppHandle) -> Result<Connection, String> {
    let path = database_path(app)?;
    let connection =
        Connection::open(path).map_err(|error| format!("failed to open database: {error}"))?;

    connection
        .execute_batch(
            "
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;
            ",
        )
        .map_err(|error| format!("failed to configure database: {error}"))?;

    Ok(connection)
}

pub(crate) fn create_schema(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS course_packs (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                language TEXT NOT NULL DEFAULT 'en',
                daily_new_target INTEGER NOT NULL DEFAULT 20 CHECK (daily_new_target >= 0),
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS lessons (
                id TEXT PRIMARY KEY,
                course_pack_id TEXT NOT NULL,
                title TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (course_pack_id) REFERENCES course_packs(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_lessons_course_pack_sort
                ON lessons(course_pack_id, sort_order);

            CREATE TABLE IF NOT EXISTS sentence_items (
                id TEXT PRIMARY KEY,
                lesson_id TEXT NOT NULL,
                english TEXT NOT NULL,
                chinese TEXT NOT NULL,
                phonetic TEXT NOT NULL DEFAULT '',
                note TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'new' CHECK (status IN ('new', 'learning', 'mastered')),
                favorite INTEGER NOT NULL DEFAULT 0 CHECK (favorite IN (0, 1)),
                show_count INTEGER NOT NULL DEFAULT 0 CHECK (show_count >= 0),
                review_count INTEGER NOT NULL DEFAULT 0 CHECK (review_count >= 0),
                error_count INTEGER NOT NULL DEFAULT 0 CHECK (error_count >= 0),
                next_review_at TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (lesson_id) REFERENCES lessons(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_sentence_items_lesson
                ON sentence_items(lesson_id);

            CREATE INDEX IF NOT EXISTS idx_sentence_items_next_review
                ON sentence_items(next_review_at);

            CREATE TABLE IF NOT EXISTS review_logs (
                id TEXT PRIMARY KEY,
                sentence_id TEXT NOT NULL,
                user_answer TEXT NOT NULL,
                is_correct INTEGER NOT NULL CHECK (is_correct IN (0, 1)),
                wrong_indexes TEXT NOT NULL DEFAULT '[]',
                rating TEXT NOT NULL CHECK (rating IN ('again', 'hard', 'good', 'easy')),
                interval_minutes INTEGER NOT NULL CHECK (interval_minutes >= 0),
                reviewed_at TEXT NOT NULL,
                FOREIGN KEY (sentence_id) REFERENCES sentence_items(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_review_logs_sentence
                ON review_logs(sentence_id);

            CREATE INDEX IF NOT EXISTS idx_review_logs_reviewed_at
                ON review_logs(reviewed_at);

            CREATE TABLE IF NOT EXISTS daily_stats (
                date TEXT PRIMARY KEY,
                new_count INTEGER NOT NULL DEFAULT 0 CHECK (new_count >= 0),
                review_count INTEGER NOT NULL DEFAULT 0 CHECK (review_count >= 0),
                study_minutes INTEGER NOT NULL DEFAULT 0 CHECK (study_minutes >= 0),
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS vocabulary_books (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                source TEXT NOT NULL DEFAULT 'markdown',
                imported_at TEXT NOT NULL,
                raw_text TEXT NOT NULL DEFAULT '',
                note TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS vocabulary_items (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL,
                normalized_text TEXT NOT NULL UNIQUE,
                primary_meaning TEXT NOT NULL,
                phonetic TEXT NOT NULL DEFAULT '',
                part_of_speech TEXT NOT NULL DEFAULT '',
                example TEXT NOT NULL DEFAULT '',
                example_cn TEXT NOT NULL DEFAULT '',
                roots TEXT NOT NULL DEFAULT '',
                word_family TEXT NOT NULL DEFAULT '',
                synonyms TEXT NOT NULL DEFAULT '',
                antonyms TEXT NOT NULL DEFAULT '',
                memory_hint TEXT NOT NULL DEFAULT '',
                tags TEXT NOT NULL DEFAULT '',
                difficulty TEXT NOT NULL DEFAULT '',
                familiarity INTEGER NOT NULL DEFAULT 0 CHECK (familiarity >= 0 AND familiarity <= 100),
                weak_score INTEGER NOT NULL DEFAULT 0 CHECK (weak_score >= 0),
                review_count INTEGER NOT NULL DEFAULT 0 CHECK (review_count >= 0),
                wrong_count INTEGER NOT NULL DEFAULT 0 CHECK (wrong_count >= 0),
                source_book_id TEXT,
                last_reviewed_at TEXT,
                next_review_at TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (source_book_id) REFERENCES vocabulary_books(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS vocabulary_book_items (
                id TEXT PRIMARY KEY,
                book_id TEXT NOT NULL,
                vocabulary_item_id TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
                meaning_snapshot TEXT NOT NULL DEFAULT '',
                source_line INTEGER NOT NULL DEFAULT 0 CHECK (source_line >= 0),
                imported_snapshot TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (book_id) REFERENCES vocabulary_books(id) ON DELETE CASCADE,
                FOREIGN KEY (vocabulary_item_id) REFERENCES vocabulary_items(id) ON DELETE CASCADE,
                UNIQUE (book_id, vocabulary_item_id)
            );

            CREATE TABLE IF NOT EXISTS vocabulary_reviews (
                id TEXT PRIMARY KEY,
                vocabulary_item_id TEXT NOT NULL,
                mode TEXT NOT NULL DEFAULT 'card',
                rating TEXT NOT NULL CHECK (rating IN ('again', 'hard', 'good', 'easy')),
                reviewed_at TEXT NOT NULL,
                next_review_at TEXT NOT NULL,
                FOREIGN KEY (vocabulary_item_id) REFERENCES vocabulary_items(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS vocabulary_memory_states (
                id TEXT PRIMARY KEY,
                vocabulary_item_id TEXT NOT NULL,
                skill_type TEXT NOT NULL CHECK (skill_type IN ('recognition', 'production', 'listening')),
                state TEXT NOT NULL DEFAULT 'new' CHECK (state IN ('new', 'learning', 'review', 'relearning', 'mastered')),
                stability REAL NOT NULL DEFAULT 0.0 CHECK (stability >= 0.0),
                difficulty REAL NOT NULL DEFAULT 5.0 CHECK (difficulty >= 1.0 AND difficulty <= 10.0),
                retrievability REAL NOT NULL DEFAULT 0.0 CHECK (retrievability >= 0.0 AND retrievability <= 1.0),
                interval_days REAL NOT NULL DEFAULT 0.0 CHECK (interval_days >= 0.0),
                due_at TEXT,
                last_reviewed_at TEXT,
                lapse_count INTEGER NOT NULL DEFAULT 0 CHECK (lapse_count >= 0),
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (vocabulary_item_id) REFERENCES vocabulary_items(id) ON DELETE CASCADE,
                UNIQUE (vocabulary_item_id, skill_type)
            );

            CREATE TABLE IF NOT EXISTS sentence_vocabulary_links (
                id TEXT PRIMARY KEY,
                sentence_id TEXT NOT NULL,
                vocabulary_item_id TEXT NOT NULL,
                matched_text TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (sentence_id) REFERENCES sentence_items(id) ON DELETE CASCADE,
                FOREIGN KEY (vocabulary_item_id) REFERENCES vocabulary_items(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_vocabulary_book_items_book
                ON vocabulary_book_items(book_id, sort_order);

            CREATE INDEX IF NOT EXISTS idx_vocabulary_book_items_item
                ON vocabulary_book_items(vocabulary_item_id);

            CREATE INDEX IF NOT EXISTS idx_vocabulary_items_due
                ON vocabulary_items(next_review_at, weak_score);

            CREATE INDEX IF NOT EXISTS idx_vocabulary_reviews_item
                ON vocabulary_reviews(vocabulary_item_id, reviewed_at);

            CREATE INDEX IF NOT EXISTS idx_vocabulary_memory_due
                ON vocabulary_memory_states(skill_type, due_at, retrievability);

            CREATE INDEX IF NOT EXISTS idx_sentence_vocabulary_links_sentence
                ON sentence_vocabulary_links(sentence_id);

            CREATE TABLE IF NOT EXISTS sync_settings (
                id TEXT PRIMARY KEY CHECK (id = 'default'),
                provider TEXT NOT NULL DEFAULT 'jianguoyun_webdav',
                base_url TEXT NOT NULL DEFAULT 'https://dav.jianguoyun.com/dav/',
                username TEXT NOT NULL DEFAULT '',
                remote_path TEXT NOT NULL DEFAULT '/MomoLite/sync/',
                device_id TEXT NOT NULL DEFAULT '',
                auto_sync_enabled INTEGER NOT NULL DEFAULT 0 CHECK (auto_sync_enabled IN (0, 1)),
                last_sync_at TEXT,
                sync_status TEXT NOT NULL DEFAULT 'idle',
                last_error TEXT NOT NULL DEFAULT '',
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS llm_settings (
                id TEXT PRIMARY KEY CHECK (id = 'default'),
                provider TEXT NOT NULL DEFAULT 'gpt2',
                base_url TEXT NOT NULL DEFAULT 'https://way.ydata.vip/v1',
                model TEXT NOT NULL DEFAULT 'gpt-5.4',
                wire_api TEXT NOT NULL DEFAULT 'responses',
                reasoning_effort TEXT NOT NULL DEFAULT 'high',
                disable_response_storage INTEGER NOT NULL DEFAULT 1 CHECK (disable_response_storage IN (0, 1)),
                prompt_version TEXT NOT NULL DEFAULT 'momolite-scene-v1',
                temperature REAL NOT NULL DEFAULT 0.4,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS llm_profiles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                provider TEXT NOT NULL DEFAULT 'gpt2',
                base_url TEXT NOT NULL DEFAULT 'https://way.ydata.vip/v1',
                model TEXT NOT NULL DEFAULT 'gpt-5.4',
                wire_api TEXT NOT NULL DEFAULT 'responses',
                reasoning_effort TEXT NOT NULL DEFAULT 'high',
                disable_response_storage INTEGER NOT NULL DEFAULT 1 CHECK (disable_response_storage IN (0, 1)),
                prompt_version TEXT NOT NULL DEFAULT 'momolite-scene-v1',
                temperature REAL NOT NULL DEFAULT 0.4,
                is_default INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS learning_plan_settings (
                id TEXT PRIMARY KEY CHECK (id = 'default'),
                intensity TEXT NOT NULL DEFAULT 'standard' CHECK (intensity IN ('light', 'standard', 'intensive')),
                desired_retention REAL NOT NULL DEFAULT 0.90 CHECK (desired_retention > 0.0 AND desired_retention < 1.0),
                daily_new_target INTEGER NOT NULL DEFAULT 25 CHECK (daily_new_target >= 0),
                daily_review_limit INTEGER NOT NULL DEFAULT 160 CHECK (daily_review_limit >= 0),
                scene_lessons_target INTEGER NOT NULL DEFAULT 3 CHECK (scene_lessons_target >= 0),
                recovery_days INTEGER NOT NULL DEFAULT 3 CHECK (recovery_days >= 1),
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS daily_learning_plans (
                id TEXT PRIMARY KEY,
                plan_date TEXT NOT NULL UNIQUE,
                intensity TEXT NOT NULL,
                new_word_target INTEGER NOT NULL DEFAULT 0,
                review_limit INTEGER NOT NULL DEFAULT 0,
                scene_lesson_target INTEGER NOT NULL DEFAULT 0,
                due_count INTEGER NOT NULL DEFAULT 0,
                weak_count INTEGER NOT NULL DEFAULT 0,
                backlog_count INTEGER NOT NULL DEFAULT 0,
                plan_json TEXT NOT NULL DEFAULT '{}',
                explanation TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS scene_generation_batches (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                course_pack_id TEXT,
                selected_topics TEXT NOT NULL DEFAULT '[]',
                word_source TEXT NOT NULL DEFAULT 'smart',
                core_word_count INTEGER NOT NULL DEFAULT 0,
                support_word_count INTEGER NOT NULL DEFAULT 0,
                planned_scene_count INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'planned' CHECK (status IN ('planned', 'generating', 'succeeded', 'partial', 'failed')),
                coverage_summary TEXT NOT NULL DEFAULT '{}',
                error_message TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (course_pack_id) REFERENCES course_packs(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS scene_generation_plans (
                id TEXT PRIMARY KEY,
                batch_id TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
                title TEXT NOT NULL,
                topic TEXT NOT NULL,
                core_word_ids TEXT NOT NULL DEFAULT '[]',
                support_word_ids TEXT NOT NULL DEFAULT '[]',
                core_words_snapshot TEXT NOT NULL DEFAULT '[]',
                support_words_snapshot TEXT NOT NULL DEFAULT '[]',
                status TEXT NOT NULL DEFAULT 'planned' CHECK (status IN ('planned', 'generating', 'succeeded', 'failed')),
                generated_scene_id TEXT,
                error_message TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (batch_id) REFERENCES scene_generation_batches(id) ON DELETE CASCADE,
                FOREIGN KEY (generated_scene_id) REFERENCES generated_scenes(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS generated_scenes (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                scenario TEXT NOT NULL DEFAULT '',
                prompt_version TEXT NOT NULL,
                model TEXT NOT NULL,
                status TEXT NOT NULL CHECK (status IN ('generating', 'succeeded', 'failed')),
                target_words_snapshot TEXT NOT NULL DEFAULT '[]',
                request_json TEXT NOT NULL DEFAULT '{}',
                response_json TEXT NOT NULL DEFAULT '{}',
                error_message TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS scene_lines (
                id TEXT PRIMARY KEY,
                scene_id TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
                speaker TEXT NOT NULL DEFAULT '',
                english TEXT NOT NULL,
                chinese TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (scene_id) REFERENCES generated_scenes(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS scene_vocabulary_links (
                id TEXT PRIMARY KEY,
                scene_id TEXT NOT NULL,
                scene_line_id TEXT,
                vocabulary_item_id TEXT,
                matched_text TEXT NOT NULL DEFAULT '',
                normalized_target TEXT NOT NULL DEFAULT '',
                start_offset INTEGER NOT NULL DEFAULT -1,
                end_offset INTEGER NOT NULL DEFAULT -1,
                occurrence_order INTEGER NOT NULL DEFAULT 0 CHECK (occurrence_order >= 0),
                created_at TEXT NOT NULL,
                FOREIGN KEY (scene_id) REFERENCES generated_scenes(id) ON DELETE CASCADE,
                FOREIGN KEY (scene_line_id) REFERENCES scene_lines(id) ON DELETE CASCADE,
                FOREIGN KEY (vocabulary_item_id) REFERENCES vocabulary_items(id) ON DELETE SET NULL
            );

            CREATE INDEX IF NOT EXISTS idx_generated_scenes_updated
                ON generated_scenes(updated_at);

            CREATE INDEX IF NOT EXISTS idx_scene_generation_plans_batch
                ON scene_generation_plans(batch_id, sort_order);

            CREATE INDEX IF NOT EXISTS idx_scene_lines_scene
                ON scene_lines(scene_id, sort_order);

            CREATE INDEX IF NOT EXISTS idx_scene_vocabulary_links_scene
                ON scene_vocabulary_links(scene_id, occurrence_order);
            ",
        )
        .map_err(|error| format!("failed to create database schema: {error}"))?;

    add_column_if_missing(
        connection,
        "generated_scenes",
        "batch_id",
        "ALTER TABLE generated_scenes ADD COLUMN batch_id TEXT",
    )?;
    add_column_if_missing(
        connection,
        "generated_scenes",
        "planned_course_pack_id",
        "ALTER TABLE generated_scenes ADD COLUMN planned_course_pack_id TEXT",
    )?;
    add_column_if_missing(
        connection,
        "generated_scenes",
        "lesson_id",
        "ALTER TABLE generated_scenes ADD COLUMN lesson_id TEXT",
    )?;
    add_column_if_missing(
        connection,
        "generated_scenes",
        "coverage_json",
        "ALTER TABLE generated_scenes ADD COLUMN coverage_json TEXT NOT NULL DEFAULT '{}'",
    )?;
    add_column_if_missing(
        connection,
        "generated_scenes",
        "is_added_to_course",
        "ALTER TABLE generated_scenes ADD COLUMN is_added_to_course INTEGER NOT NULL DEFAULT 0 CHECK (is_added_to_course IN (0, 1))",
    )?;
    add_column_if_missing(
        connection,
        "vocabulary_reviews",
        "skill_type",
        "ALTER TABLE vocabulary_reviews ADD COLUMN skill_type TEXT NOT NULL DEFAULT 'recognition'",
    )?;
    add_column_if_missing(
        connection,
        "vocabulary_reviews",
        "stability_before",
        "ALTER TABLE vocabulary_reviews ADD COLUMN stability_before REAL NOT NULL DEFAULT 0.0",
    )?;
    add_column_if_missing(
        connection,
        "vocabulary_reviews",
        "stability_after",
        "ALTER TABLE vocabulary_reviews ADD COLUMN stability_after REAL NOT NULL DEFAULT 0.0",
    )?;
    add_column_if_missing(
        connection,
        "vocabulary_reviews",
        "difficulty_before",
        "ALTER TABLE vocabulary_reviews ADD COLUMN difficulty_before REAL NOT NULL DEFAULT 5.0",
    )?;
    add_column_if_missing(
        connection,
        "vocabulary_reviews",
        "difficulty_after",
        "ALTER TABLE vocabulary_reviews ADD COLUMN difficulty_after REAL NOT NULL DEFAULT 5.0",
    )?;
    add_column_if_missing(
        connection,
        "vocabulary_reviews",
        "retrievability_before",
        "ALTER TABLE vocabulary_reviews ADD COLUMN retrievability_before REAL NOT NULL DEFAULT 0.0",
    )?;
    add_column_if_missing(
        connection,
        "vocabulary_reviews",
        "retrievability_after",
        "ALTER TABLE vocabulary_reviews ADD COLUMN retrievability_after REAL NOT NULL DEFAULT 0.0",
    )?;

    seed_default_llm_profile(connection)?;
    seed_learning_plan_settings(connection)?;
    migrate_to_normalized_schema(connection)?;
    merge_duplicate_lessons(connection)?;

    connection
        .execute_batch(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS idx_lessons_course_pack_title
                ON lessons(course_pack_id, title);

            CREATE INDEX IF NOT EXISTS idx_sentence_items_lesson
                ON sentence_items(lesson_id);

            CREATE INDEX IF NOT EXISTS idx_sentence_items_next_review
                ON sentence_items(next_review_at);

            CREATE INDEX IF NOT EXISTS idx_review_logs_sentence
                ON review_logs(sentence_id);

            CREATE INDEX IF NOT EXISTS idx_review_logs_reviewed_at
                ON review_logs(reviewed_at);
            ",
        )
        .map_err(|error| format!("failed to create database indexes: {error}"))?;

    Ok(())
}

fn migrate_to_normalized_schema(connection: &Connection) -> Result<(), String> {
    if table_has_column(connection, "sentence_items", "course_pack_id")? {
        connection
            .execute_batch(
                "
                PRAGMA foreign_keys = OFF;

                DROP INDEX IF EXISTS idx_sentence_items_course_pack;
                DROP INDEX IF EXISTS idx_sentence_items_lesson;
                DROP INDEX IF EXISTS idx_sentence_items_next_review;

                ALTER TABLE sentence_items RENAME TO sentence_items_old;

                CREATE TABLE sentence_items (
                    id TEXT PRIMARY KEY,
                    lesson_id TEXT NOT NULL,
                    english TEXT NOT NULL,
                    chinese TEXT NOT NULL,
                    phonetic TEXT NOT NULL DEFAULT '',
                    note TEXT NOT NULL DEFAULT '',
                    status TEXT NOT NULL DEFAULT 'new' CHECK (status IN ('new', 'learning', 'mastered')),
                    favorite INTEGER NOT NULL DEFAULT 0 CHECK (favorite IN (0, 1)),
                    show_count INTEGER NOT NULL DEFAULT 0 CHECK (show_count >= 0),
                    review_count INTEGER NOT NULL DEFAULT 0 CHECK (review_count >= 0),
                    error_count INTEGER NOT NULL DEFAULT 0 CHECK (error_count >= 0),
                    next_review_at TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    FOREIGN KEY (lesson_id) REFERENCES lessons(id) ON DELETE CASCADE
                );

                INSERT INTO sentence_items
                    (
                        id,
                        lesson_id,
                        english,
                        chinese,
                        phonetic,
                        note,
                        status,
                        favorite,
                        show_count,
                        review_count,
                        error_count,
                        next_review_at,
                        created_at,
                        updated_at
                    )
                SELECT
                    id,
                    lesson_id,
                    english,
                    chinese,
                    phonetic,
                    note,
                    status,
                    favorite,
                    show_count,
                    review_count,
                    error_count,
                    next_review_at,
                    created_at,
                    updated_at
                FROM sentence_items_old;

                DROP TABLE sentence_items_old;
                PRAGMA foreign_keys = ON;
                ",
            )
            .map_err(|error| format!("failed to normalize sentence_items: {error}"))?;
    }

    if table_has_column(connection, "review_logs", "course_pack_id")? {
        connection
            .execute_batch(
                "
                PRAGMA foreign_keys = OFF;

                DROP INDEX IF EXISTS idx_review_logs_sentence;
                DROP INDEX IF EXISTS idx_review_logs_reviewed_at;

                ALTER TABLE review_logs RENAME TO review_logs_old;

                CREATE TABLE review_logs (
                    id TEXT PRIMARY KEY,
                    sentence_id TEXT NOT NULL,
                    user_answer TEXT NOT NULL,
                    is_correct INTEGER NOT NULL CHECK (is_correct IN (0, 1)),
                    wrong_indexes TEXT NOT NULL DEFAULT '[]',
                    rating TEXT NOT NULL CHECK (rating IN ('again', 'hard', 'good', 'easy')),
                    interval_minutes INTEGER NOT NULL CHECK (interval_minutes >= 0),
                    reviewed_at TEXT NOT NULL,
                    FOREIGN KEY (sentence_id) REFERENCES sentence_items(id) ON DELETE CASCADE
                );

                INSERT INTO review_logs
                    (
                        id,
                        sentence_id,
                        user_answer,
                        is_correct,
                        wrong_indexes,
                        rating,
                        interval_minutes,
                        reviewed_at
                    )
                SELECT
                    id,
                    sentence_id,
                    user_answer,
                    is_correct,
                    wrong_indexes,
                    rating,
                    interval_minutes,
                    reviewed_at
                FROM review_logs_old;

                DROP TABLE review_logs_old;
                PRAGMA foreign_keys = ON;
                ",
            )
            .map_err(|error| format!("failed to normalize review_logs: {error}"))?;
    }

    Ok(())
}

fn merge_duplicate_lessons(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "
            UPDATE sentence_items
            SET lesson_id = (
                SELECT canonical.id
                FROM lessons duplicate
                JOIN lessons canonical
                    ON canonical.course_pack_id = duplicate.course_pack_id
                    AND canonical.title = duplicate.title
                WHERE duplicate.id = sentence_items.lesson_id
                ORDER BY canonical.created_at ASC, canonical.id ASC
                LIMIT 1
            )
            WHERE lesson_id IN (
                SELECT duplicate.id
                FROM lessons duplicate
                WHERE duplicate.id <> (
                    SELECT canonical.id
                    FROM lessons canonical
                    WHERE canonical.course_pack_id = duplicate.course_pack_id
                        AND canonical.title = duplicate.title
                    ORDER BY canonical.created_at ASC, canonical.id ASC
                    LIMIT 1
                )
            );

            DELETE FROM lessons
            WHERE id <> (
                SELECT canonical.id
                FROM lessons canonical
                WHERE canonical.course_pack_id = lessons.course_pack_id
                    AND canonical.title = lessons.title
                ORDER BY canonical.created_at ASC, canonical.id ASC
                LIMIT 1
            );
            ",
        )
        .map_err(|error| format!("failed to merge duplicate lessons: {error}"))?;

    Ok(())
}

#[tauri::command]
pub fn init_database(app: AppHandle) -> Result<String, String> {
    let path = database_path(&app)?;
    let connection = open_database(&app)?;
    create_schema(&connection)?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn load_app_state(app: AppHandle) -> Result<AppState, String> {
    let connection = open_database(&app)?;
    create_schema(&connection)?;

    let mut course_statement = connection
        .prepare(
            "
            SELECT id, name, language, daily_new_target, created_at, updated_at
            FROM course_packs
            ORDER BY created_at DESC
            ",
        )
        .map_err(|error| format!("failed to prepare course query: {error}"))?;
    let course_packs = course_statement
        .query_map([], |row| {
            Ok(CoursePack {
                id: row.get(0)?,
                name: row.get(1)?,
                language: row.get(2)?,
                daily_new_target: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|error| format!("failed to query courses: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect courses: {error}"))?;

    let mut sentence_statement = connection
        .prepare(
            "
            SELECT
                s.id,
                l.course_pack_id,
                l.title,
                s.english,
                s.chinese,
                s.phonetic,
                s.note,
                s.status,
                s.favorite,
                s.show_count,
                s.review_count,
                s.error_count,
                s.next_review_at,
                s.created_at,
                s.updated_at
            FROM sentence_items s
            JOIN lessons l ON l.id = s.lesson_id
            ORDER BY l.sort_order ASC, s.created_at ASC
            ",
        )
        .map_err(|error| format!("failed to prepare sentence query: {error}"))?;
    let sentences = sentence_statement
        .query_map([], |row| {
            let favorite: i64 = row.get(8)?;
            Ok(SentenceItem {
                id: row.get(0)?,
                course_pack_id: row.get(1)?,
                lesson_title: row.get(2)?,
                english: row.get(3)?,
                chinese: row.get(4)?,
                phonetic: row.get(5)?,
                note: row.get(6)?,
                status: row.get(7)?,
                favorite: favorite != 0,
                show_count: row.get(9)?,
                review_count: row.get(10)?,
                error_count: row.get(11)?,
                next_review_at: row.get(12)?,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })
        .map_err(|error| format!("failed to query sentences: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect sentences: {error}"))?;

    let mut review_statement = connection
        .prepare(
            "
            SELECT
                r.id,
                r.sentence_id,
                l.course_pack_id,
                r.user_answer,
                r.is_correct,
                r.wrong_indexes,
                r.rating,
                r.interval_minutes,
                r.reviewed_at
            FROM review_logs r
            JOIN sentence_items s ON s.id = r.sentence_id
            JOIN lessons l ON l.id = s.lesson_id
            ORDER BY r.reviewed_at ASC
            ",
        )
        .map_err(|error| format!("failed to prepare review query: {error}"))?;
    let reviews = review_statement
        .query_map([], |row| {
            let is_correct: i64 = row.get(4)?;
            let wrong_indexes_json: String = row.get(5)?;
            let wrong_indexes =
                serde_json::from_str::<Vec<usize>>(&wrong_indexes_json).unwrap_or_default();

            Ok(ReviewLog {
                id: row.get(0)?,
                sentence_id: row.get(1)?,
                course_pack_id: row.get(2)?,
                user_answer: row.get(3)?,
                is_correct: is_correct != 0,
                wrong_indexes,
                rating: row.get(6)?,
                interval_minutes: row.get(7)?,
                reviewed_at: row.get(8)?,
            })
        })
        .map_err(|error| format!("failed to query reviews: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect reviews: {error}"))?;

    let mut stats_statement = connection
        .prepare(
            "
            SELECT date, new_count, review_count, study_minutes
            FROM daily_stats
            ORDER BY date ASC
            ",
        )
        .map_err(|error| format!("failed to prepare stats query: {error}"))?;
    let stats_pairs = stats_statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                DailyStats {
                    new_count: row.get(1)?,
                    review_count: row.get(2)?,
                    study_minutes: row.get(3)?,
                },
            ))
        })
        .map_err(|error| format!("failed to query stats: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect stats: {error}"))?;
    let stats = stats_pairs.into_iter().collect::<HashMap<_, _>>();

    let active_course_pack_id = course_packs
        .first()
        .map(|course| course.id.clone())
        .unwrap_or_default();

    Ok(AppState {
        active_course_pack_id,
        course_packs,
        sentences,
        reviews,
        stats,
    })
}

#[tauri::command]
pub fn list_course_packs(app: AppHandle) -> Result<Vec<CoursePack>, String> {
    let connection = open_database(&app)?;
    create_schema(&connection)?;
    let mut statement = connection
        .prepare(
            "
            SELECT id, name, language, daily_new_target, created_at, updated_at
            FROM course_packs
            ORDER BY created_at DESC
            ",
        )
        .map_err(|error| format!("failed to prepare course list: {error}"))?;

    let course_packs = statement
        .query_map([], |row| {
            Ok(CoursePack {
                id: row.get(0)?,
                name: row.get(1)?,
                language: row.get(2)?,
                daily_new_target: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|error| format!("failed to query course list: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect course list: {error}"))?;

    Ok(course_packs)
}

#[tauri::command]
pub fn create_course_pack(app: AppHandle, course: NewCoursePack) -> Result<(), String> {
    let connection = open_database(&app)?;
    create_schema(&connection)?;
    connection
        .execute(
            "
            INSERT OR IGNORE INTO course_packs
                (id, name, language, daily_new_target, created_at, updated_at)
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6)
            ",
            params![
                course.id,
                course.name,
                course.language,
                course.daily_new_target,
                course.created_at,
                course.created_at
            ],
        )
        .map_err(|error| format!("failed to create course pack: {error}"))?;
    Ok(())
}

#[tauri::command]
pub fn delete_course_pack(app: AppHandle, course_pack_id: String) -> Result<(), String> {
    let connection = open_database(&app)?;
    create_schema(&connection)?;
    delete_course_pack_on_connection(&connection, &course_pack_id)
}

fn delete_course_pack_on_connection(
    connection: &Connection,
    course_pack_id: &str,
) -> Result<(), String> {
    connection
        .execute(
            "
            DELETE FROM course_packs
            WHERE id = ?1
            ",
            params![course_pack_id],
        )
        .map_err(|error| format!("failed to delete course pack: {error}"))?;
    Ok(())
}

#[tauri::command]
pub fn list_lessons(app: AppHandle, course_pack_id: String) -> Result<Vec<NewLesson>, String> {
    let connection = open_database(&app)?;
    create_schema(&connection)?;
    let mut statement = connection
        .prepare(
            "
            SELECT id, course_pack_id, title, sort_order, created_at
            FROM lessons
            WHERE course_pack_id = ?1
            ORDER BY sort_order ASC
            ",
        )
        .map_err(|error| format!("failed to prepare lesson list: {error}"))?;

    let lessons = statement
        .query_map(params![course_pack_id], |row| {
            Ok(NewLesson {
                id: row.get(0)?,
                course_pack_id: row.get(1)?,
                title: row.get(2)?,
                sort_order: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|error| format!("failed to query lesson list: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect lesson list: {error}"))?;

    Ok(lessons)
}

#[tauri::command]
pub fn create_lesson(app: AppHandle, lesson: NewLesson) -> Result<(), String> {
    let connection = open_database(&app)?;
    create_schema(&connection)?;
    connection
        .execute(
            "
            INSERT OR IGNORE INTO lessons
                (id, course_pack_id, title, sort_order, created_at, updated_at)
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6)
            ",
            params![
                lesson.id,
                lesson.course_pack_id,
                lesson.title,
                lesson.sort_order,
                lesson.created_at,
                lesson.created_at
            ],
        )
        .map_err(|error| format!("failed to create lesson: {error}"))?;
    Ok(())
}

#[tauri::command]
pub fn list_sentences(app: AppHandle, course_pack_id: String) -> Result<Vec<SentenceItem>, String> {
    let connection = open_database(&app)?;
    create_schema(&connection)?;
    let mut statement = connection
        .prepare(
            "
            SELECT
                s.id,
                l.course_pack_id,
                l.title,
                s.english,
                s.chinese,
                s.phonetic,
                s.note,
                s.status,
                s.favorite,
                s.show_count,
                s.review_count,
                s.error_count,
                s.next_review_at,
                s.created_at,
                s.updated_at
            FROM sentence_items s
            JOIN lessons l ON l.id = s.lesson_id
            WHERE l.course_pack_id = ?1
            ORDER BY l.sort_order ASC, s.created_at ASC
            ",
        )
        .map_err(|error| format!("failed to prepare sentence list: {error}"))?;

    let sentences = statement
        .query_map(params![course_pack_id], |row| {
            let favorite: i64 = row.get(8)?;
            Ok(SentenceItem {
                id: row.get(0)?,
                course_pack_id: row.get(1)?,
                lesson_title: row.get(2)?,
                english: row.get(3)?,
                chinese: row.get(4)?,
                phonetic: row.get(5)?,
                note: row.get(6)?,
                status: row.get(7)?,
                favorite: favorite != 0,
                show_count: row.get(9)?,
                review_count: row.get(10)?,
                error_count: row.get(11)?,
                next_review_at: row.get(12)?,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })
        .map_err(|error| format!("failed to query sentence list: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect sentence list: {error}"))?;

    Ok(sentences)
}

#[tauri::command]
pub fn create_sentence(app: AppHandle, sentence: NewSentenceItem) -> Result<(), String> {
    let connection = open_database(&app)?;
    create_schema(&connection)?;
    connection
        .execute(
            "
            INSERT OR IGNORE INTO sentence_items
                (
                    id,
                    lesson_id,
                    english,
                    chinese,
                    phonetic,
                    note,
                    status,
                    favorite,
                    show_count,
                    review_count,
                    error_count,
                    next_review_at,
                    created_at,
                    updated_at
                )
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, 'new', 0, 0, 0, 0, NULL, ?7, ?8)
            ",
            params![
                sentence.id,
                sentence.lesson_id,
                sentence.english,
                sentence.chinese,
                sentence.phonetic,
                sentence.note,
                sentence.created_at,
                sentence.created_at
            ],
        )
        .map_err(|error| format!("failed to create sentence: {error}"))?;
    Ok(())
}

#[tauri::command]
pub fn delete_sentence(app: AppHandle, sentence_id: String) -> Result<(), String> {
    let connection = open_database(&app)?;
    create_schema(&connection)?;
    delete_sentence_on_connection(&connection, &sentence_id)
}

fn delete_sentence_on_connection(connection: &Connection, sentence_id: &str) -> Result<(), String> {
    connection
        .execute(
            "
            DELETE FROM sentence_items
            WHERE id = ?1
            ",
            params![sentence_id],
        )
        .map_err(|error| format!("failed to delete sentence: {error}"))?;
    Ok(())
}

fn vocabulary_item_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<VocabularyItem> {
    Ok(VocabularyItem {
        id: row.get(0)?,
        text: row.get(1)?,
        normalized_text: row.get(2)?,
        primary_meaning: row.get(3)?,
        phonetic: row.get(4)?,
        part_of_speech: row.get(5)?,
        example: row.get(6)?,
        example_cn: row.get(7)?,
        roots: row.get(8)?,
        word_family: row.get(9)?,
        synonyms: row.get(10)?,
        antonyms: row.get(11)?,
        memory_hint: row.get(12)?,
        tags: row.get(13)?,
        difficulty: row.get(14)?,
        familiarity: row.get(15)?,
        weak_score: row.get(16)?,
        review_count: row.get(17)?,
        wrong_count: row.get(18)?,
        last_reviewed_at: row.get(19)?,
        next_review_at: row.get(20)?,
        created_at: row.get(21)?,
        updated_at: row.get(22)?,
    })
}

fn vocabulary_item_select_sql() -> &'static str {
    "
    SELECT
        vocabulary_items.id,
        vocabulary_items.text,
        vocabulary_items.normalized_text,
        vocabulary_items.primary_meaning,
        vocabulary_items.phonetic,
        vocabulary_items.part_of_speech,
        vocabulary_items.example,
        vocabulary_items.example_cn,
        vocabulary_items.roots,
        vocabulary_items.word_family,
        vocabulary_items.synonyms,
        vocabulary_items.antonyms,
        vocabulary_items.memory_hint,
        vocabulary_items.tags,
        vocabulary_items.difficulty,
        vocabulary_items.familiarity,
        vocabulary_items.weak_score,
        vocabulary_items.review_count,
        vocabulary_items.wrong_count,
        vocabulary_items.last_reviewed_at,
        vocabulary_items.next_review_at,
        vocabulary_items.created_at,
        vocabulary_items.updated_at
    FROM vocabulary_items
    "
}

fn get_vocabulary_item_on_connection(
    connection: &Connection,
    vocabulary_item_id: &str,
) -> Result<VocabularyItem, String> {
    connection
        .query_row(
            &format!("{} WHERE id = ?1", vocabulary_item_select_sql()),
            params![vocabulary_item_id],
            vocabulary_item_from_row,
        )
        .map_err(|error| format!("failed to query vocabulary item: {error}"))
}

fn vocabulary_book_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<VocabularyBook> {
    Ok(VocabularyBook {
        id: row.get(0)?,
        name: row.get(1)?,
        source: row.get(2)?,
        imported_at: row.get(3)?,
        note: row.get(4)?,
        item_count: row.get(5)?,
    })
}

fn list_vocabulary_books_on_connection(
    connection: &Connection,
) -> Result<Vec<VocabularyBook>, String> {
    let mut statement = connection
        .prepare(
            "
            SELECT
                b.id,
                b.name,
                b.source,
                b.imported_at,
                b.note,
                COUNT(bi.vocabulary_item_id) AS item_count
            FROM vocabulary_books b
            LEFT JOIN vocabulary_book_items bi ON bi.book_id = b.id
            GROUP BY b.id
            ORDER BY b.imported_at DESC, b.created_at DESC
            ",
        )
        .map_err(|error| format!("failed to prepare vocabulary books: {error}"))?;

    let books = statement
        .query_map([], vocabulary_book_from_row)
        .map_err(|error| format!("failed to query vocabulary books: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect vocabulary books: {error}"))?;
    Ok(books)
}

#[tauri::command]
pub fn list_vocabulary_books(app: AppHandle) -> Result<Vec<VocabularyBook>, String> {
    let connection = open_database(&app)?;
    create_schema(&connection)?;
    list_vocabulary_books_on_connection(&connection)
}

#[tauri::command]
pub fn list_vocabulary_items(app: AppHandle) -> Result<Vec<VocabularyItem>, String> {
    let connection = open_database(&app)?;
    create_schema(&connection)?;
    let mut statement = connection
        .prepare(&format!(
            "{} ORDER BY created_at DESC, text ASC",
            vocabulary_item_select_sql()
        ))
        .map_err(|error| format!("failed to prepare vocabulary items: {error}"))?;

    let items = statement
        .query_map([], vocabulary_item_from_row)
        .map_err(|error| format!("failed to query vocabulary items: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect vocabulary items: {error}"))?;
    Ok(items)
}

#[tauri::command]
pub fn get_vocabulary_book(
    app: AppHandle,
    book_id: String,
) -> Result<VocabularyBookDetail, String> {
    let connection = open_database(&app)?;
    create_schema(&connection)?;
    let book = connection
        .query_row(
            "
            SELECT
                b.id,
                b.name,
                b.source,
                b.imported_at,
                b.note,
                COUNT(bi.vocabulary_item_id) AS item_count
            FROM vocabulary_books b
            LEFT JOIN vocabulary_book_items bi ON bi.book_id = b.id
            WHERE b.id = ?1
            GROUP BY b.id
            ",
            params![book_id],
            vocabulary_book_from_row,
        )
        .map_err(|error| format!("failed to query vocabulary book: {error}"))?;

    let mut statement = connection
        .prepare(&format!(
            "
            {}
            WHERE id IN (
                SELECT vocabulary_item_id
                FROM vocabulary_book_items
                WHERE book_id = ?1
            )
            ORDER BY (
                SELECT sort_order
                FROM vocabulary_book_items
                WHERE book_id = ?1
                    AND vocabulary_item_id = vocabulary_items.id
            ) ASC
            ",
            vocabulary_item_select_sql()
        ))
        .map_err(|error| format!("failed to prepare vocabulary book detail: {error}"))?;
    let items = statement
        .query_map(params![book.id.clone()], vocabulary_item_from_row)
        .map_err(|error| format!("failed to query vocabulary book items: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect vocabulary book items: {error}"))?;

    Ok(VocabularyBookDetail { book, items })
}

#[tauri::command]
pub fn import_vocabulary_book(
    app: AppHandle,
    request: ImportVocabularyBookRequest,
) -> Result<ImportVocabularyBookResult, String> {
    let mut connection = open_database(&app)?;
    create_schema(&connection)?;

    let (parsed_title, entries, parse_skipped) = parse_vocabulary_book_markdown(&request.raw_text);
    if entries.is_empty() {
        return Err(
            "没有识别到可导入的单词。请使用 Markdown 格式：## word + 字段列表。".to_string(),
        );
    }

    let imported_at = if request.imported_at.trim().is_empty() {
        now_string()
    } else {
        request.imported_at.trim().to_string()
    };
    let book_name = if request.name.trim().is_empty() {
        parsed_title.unwrap_or_else(|| "未命名单词书".to_string())
    } else {
        request.name.trim().to_string()
    };
    let book_id = generated_id("vocab_book", 0);
    let transaction = connection
        .transaction()
        .map_err(|error| format!("failed to start vocabulary import transaction: {error}"))?;

    transaction
        .execute(
            "
            INSERT INTO vocabulary_books
                (id, name, source, imported_at, raw_text, note, created_at, updated_at)
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ",
            params![
                book_id,
                book_name,
                if request.source.trim().is_empty() {
                    "markdown"
                } else {
                    request.source.trim()
                },
                imported_at,
                request.raw_text,
                request.note,
                imported_at,
                imported_at
            ],
        )
        .map_err(|error| format!("failed to insert vocabulary book: {error}"))?;

    let mut imported_count = 0;
    let mut reused_count = 0;

    for (index, entry) in entries.iter().enumerate() {
        let normalized_text = normalize_vocabulary_text(&entry.text);
        if normalized_text.is_empty() {
            continue;
        }

        let existing_id = transaction
            .query_row(
                "SELECT id FROM vocabulary_items WHERE normalized_text = ?1",
                params![normalized_text],
                |row| row.get::<_, String>(0),
            )
            .ok();

        let vocabulary_item_id = if let Some(id) = existing_id {
            reused_count += 1;
            transaction
                .execute(
                    "
                    UPDATE vocabulary_items
                    SET
                        primary_meaning = CASE WHEN ?1 != '' THEN ?1 ELSE primary_meaning END,
                        phonetic = CASE WHEN ?2 != '' THEN ?2 ELSE phonetic END,
                        part_of_speech = CASE WHEN ?3 != '' THEN ?3 ELSE part_of_speech END,
                        example = CASE WHEN ?4 != '' THEN ?4 ELSE example END,
                        example_cn = CASE WHEN ?5 != '' THEN ?5 ELSE example_cn END,
                        roots = CASE WHEN ?6 != '' THEN ?6 ELSE roots END,
                        word_family = CASE WHEN ?7 != '' THEN ?7 ELSE word_family END,
                        synonyms = CASE WHEN ?8 != '' THEN ?8 ELSE synonyms END,
                        antonyms = CASE WHEN ?9 != '' THEN ?9 ELSE antonyms END,
                        memory_hint = CASE WHEN ?10 != '' THEN ?10 ELSE memory_hint END,
                        tags = CASE WHEN ?11 != '' THEN ?11 ELSE tags END,
                        difficulty = CASE WHEN ?12 != '' THEN ?12 ELSE difficulty END,
                        updated_at = ?13
                    WHERE id = ?14
                    ",
                    params![
                        entry.primary_meaning,
                        entry.phonetic,
                        entry.part_of_speech,
                        entry.example,
                        entry.example_cn,
                        entry.roots,
                        entry.word_family,
                        entry.synonyms,
                        entry.antonyms,
                        entry.memory_hint,
                        entry.tags,
                        entry.difficulty,
                        imported_at,
                        id
                    ],
                )
                .map_err(|error| format!("failed to update vocabulary item: {error}"))?;
            id
        } else {
            imported_count += 1;
            let id = generated_id("vocab", index + 1);
            transaction
                .execute(
                    "
                    INSERT INTO vocabulary_items
                        (
                            id,
                            text,
                            normalized_text,
                            primary_meaning,
                            phonetic,
                            part_of_speech,
                            example,
                            example_cn,
                            roots,
                            word_family,
                            synonyms,
                            antonyms,
                            memory_hint,
                            tags,
                            difficulty,
                            source_book_id,
                            created_at,
                            updated_at
                        )
                    VALUES
                        (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
                    ",
                    params![
                        id,
                        entry.text,
                        normalized_text,
                        entry.primary_meaning,
                        entry.phonetic,
                        entry.part_of_speech,
                        entry.example,
                        entry.example_cn,
                        entry.roots,
                        entry.word_family,
                        entry.synonyms,
                        entry.antonyms,
                        entry.memory_hint,
                        entry.tags,
                        entry.difficulty,
                        book_id,
                        imported_at,
                        imported_at
                    ],
                )
                .map_err(|error| format!("failed to insert vocabulary item: {error}"))?;
            id
        };

        let imported_snapshot = serde_json::to_string(entry).unwrap_or_else(|_| "{}".to_string());
        transaction
            .execute(
                "
                INSERT OR IGNORE INTO vocabulary_book_items
                    (
                        id,
                        book_id,
                        vocabulary_item_id,
                        sort_order,
                        meaning_snapshot,
                        source_line,
                        imported_snapshot,
                        created_at,
                        updated_at
                    )
                VALUES
                    (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                ",
                params![
                    generated_id("vocab_book_item", index + 1),
                    book_id,
                    vocabulary_item_id,
                    index as i64,
                    entry.primary_meaning,
                    entry.source_line,
                    imported_snapshot,
                    imported_at,
                    imported_at
                ],
            )
            .map_err(|error| format!("failed to insert vocabulary book item: {error}"))?;
    }

    transaction
        .commit()
        .map_err(|error| format!("failed to commit vocabulary import: {error}"))?;

    let book = list_vocabulary_books_on_connection(&connection)?
        .into_iter()
        .find(|book| book.id == book_id)
        .ok_or_else(|| "failed to reload imported vocabulary book".to_string())?;
    let detail = get_vocabulary_book(app, book_id)?;

    Ok(ImportVocabularyBookResult {
        book,
        items: detail.items,
        imported_count,
        reused_count,
        skipped_count: parse_skipped,
    })
}

#[tauri::command]
pub fn get_today_vocabulary_queue(
    app: AppHandle,
    now: String,
    limit: i64,
) -> Result<Vec<VocabularyItem>, String> {
    let connection = open_database(&app)?;
    create_schema(&connection)?;
    let safe_limit = if limit <= 0 { 20 } else { limit.min(100) };
    let mut statement = connection
        .prepare(&format!(
            "
            {}
            LEFT JOIN vocabulary_memory_states memory
                ON memory.vocabulary_item_id = vocabulary_items.id
                AND memory.skill_type = 'recognition'
            WHERE
                COALESCE(memory.due_at, vocabulary_items.next_review_at) IS NULL
                OR COALESCE(memory.due_at, vocabulary_items.next_review_at) <= ?1
                OR vocabulary_items.review_count = 0
                OR vocabulary_items.weak_score > 0
            ORDER BY
                CASE
                    WHEN COALESCE(memory.due_at, vocabulary_items.next_review_at) IS NULL THEN 0
                    WHEN COALESCE(memory.due_at, vocabulary_items.next_review_at) <= ?1 THEN 0
                    WHEN vocabulary_items.weak_score > 0 THEN 1
                    ELSE 2
                END ASC,
                vocabulary_items.weak_score DESC,
                COALESCE(memory.retrievability, 0.0) ASC,
                vocabulary_items.review_count ASC,
                vocabulary_items.created_at DESC
            LIMIT ?2
            ",
            vocabulary_item_select_sql()
        ))
        .map_err(|error| format!("failed to prepare vocabulary queue: {error}"))?;

    let items = statement
        .query_map(params![now, safe_limit], vocabulary_item_from_row)
        .map_err(|error| format!("failed to query vocabulary queue: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to collect vocabulary queue: {error}"))?;
    Ok(items)
}

#[tauri::command]
pub fn review_vocabulary_item(
    app: AppHandle,
    review: VocabularyReviewSubmission,
) -> Result<VocabularyItem, String> {
    let connection = open_database(&app)?;
    create_schema(&connection)?;

    let (familiarity_delta, weak_delta, wrong_increment) = match review.rating.as_str() {
        "again" => (-20, 3, 1),
        "hard" => (-5, 2, 1),
        "good" => (10, -1, 0),
        "easy" => (20, -2, 0),
        _ => return Err("invalid vocabulary rating".to_string()),
    };

    let current = get_vocabulary_item_on_connection(&connection, &review.vocabulary_item_id)?;
    let next_familiarity = clamp_score(current.familiarity + familiarity_delta, 0, 100);
    let next_weak_score = clamp_score(current.weak_score + weak_delta, 0, i64::MAX);
    let mode = if review.mode.trim().is_empty() {
        "card"
    } else {
        review.mode.trim()
    };
    let (_, memory_after) = update_vocabulary_memory(
        &connection,
        &review.vocabulary_item_id,
        "recognition",
        mode,
        &review.rating,
        &review.reviewed_at,
    )?;
    let next_review_at = memory_after
        .due_at
        .clone()
        .unwrap_or_else(|| review.next_review_at.clone());

    connection
        .execute(
            "
            UPDATE vocabulary_items
            SET
                familiarity = ?1,
                weak_score = ?2,
                review_count = review_count + 1,
                wrong_count = wrong_count + ?3,
                last_reviewed_at = ?4,
                next_review_at = ?5,
                updated_at = ?4
            WHERE id = ?6
            ",
            params![
                next_familiarity,
                next_weak_score,
                wrong_increment,
                review.reviewed_at,
                next_review_at,
                review.vocabulary_item_id
            ],
        )
        .map_err(|error| format!("failed to update vocabulary item review state: {error}"))?;

    get_vocabulary_item_on_connection(&connection, &review.vocabulary_item_id)
}

fn record_review_on_connection(
    connection: &mut Connection,
    review: ReviewSubmission,
) -> Result<(), String> {
    create_schema(connection)?;
    let transaction = connection
        .transaction()
        .map_err(|error| format!("failed to start review transaction: {error}"))?;

    let previous_status: String = transaction
        .query_row(
            "
            SELECT status
            FROM sentence_items
            WHERE id = ?1
            ",
            params![review.sentence_id],
            |row| row.get(0),
        )
        .map_err(|error| format!("failed to find sentence for review: {error}"))?;

    let wrong_indexes = serde_json::to_string(&review.wrong_indexes)
        .map_err(|error| format!("failed to serialize wrong indexes: {error}"))?;
    let timestamp = now_string();
    let next_status = if review.is_correct && matches!(review.rating.as_str(), "good" | "easy") {
        "mastered"
    } else {
        "learning"
    };
    let error_increment = if review.is_correct { 0 } else { 1 };

    transaction
        .execute(
            "
            INSERT INTO review_logs
                (
                    id,
                    sentence_id,
                    user_answer,
                    is_correct,
                    wrong_indexes,
                    rating,
                    interval_minutes,
                    reviewed_at
                )
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ",
            params![
                review.id,
                review.sentence_id,
                review.user_answer,
                if review.is_correct { 1 } else { 0 },
                wrong_indexes,
                review.rating,
                review.interval_minutes,
                review.reviewed_at
            ],
        )
        .map_err(|error| format!("failed to insert review log: {error}"))?;

    transaction
        .execute(
            "
            UPDATE sentence_items
            SET
                status = ?1,
                show_count = show_count + 1,
                review_count = review_count + 1,
                error_count = error_count + ?2,
                next_review_at = ?3,
                updated_at = ?4
            WHERE id = ?5
            ",
            params![
                next_status,
                error_increment,
                review.next_review_at,
                timestamp,
                review.sentence_id
            ],
        )
        .map_err(|error| format!("failed to update sentence review state: {error}"))?;

    let linked_vocabulary_ids = {
        let mut statement = transaction
            .prepare(
                "
                SELECT DISTINCT vocabulary_item_id
                FROM sentence_vocabulary_links
                WHERE sentence_id = ?1
                ",
            )
            .map_err(|error| format!("failed to prepare linked vocabulary query: {error}"))?;
        let linked = statement
            .query_map(params![review.sentence_id], |row| row.get::<_, String>(0))
            .map_err(|error| format!("failed to query linked vocabulary: {error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("failed to collect linked vocabulary: {error}"))?;
        linked
    };
    let vocabulary_rating = if review.is_correct {
        review.rating.as_str()
    } else {
        "again"
    };
    let weak_delta = if review.is_correct { -1 } else { 2 };
    for vocabulary_item_id in linked_vocabulary_ids {
        update_vocabulary_memory(
            &transaction,
            &vocabulary_item_id,
            "production",
            "chinese_to_english",
            vocabulary_rating,
            &review.reviewed_at,
        )?;
        transaction
            .execute(
                "
                UPDATE vocabulary_items
                SET
                    weak_score = MAX(0, weak_score + ?1),
                    wrong_count = wrong_count + ?2,
                    updated_at = ?3
                WHERE id = ?4
                ",
                params![
                    weak_delta,
                    if review.is_correct { 0 } else { 1 },
                    timestamp,
                    vocabulary_item_id
                ],
            )
            .map_err(|error| format!("failed to update linked vocabulary weakness: {error}"))?;
    }

    let new_increment = if previous_status == "new" { 1 } else { 0 };
    let date = review.reviewed_at.chars().take(10).collect::<String>();
    transaction
        .execute(
            "
            INSERT INTO daily_stats
                (date, new_count, review_count, study_minutes, updated_at)
            VALUES
                (?1, ?2, 1, 0, ?3)
            ON CONFLICT(date) DO UPDATE SET
                new_count = new_count + excluded.new_count,
                review_count = review_count + 1,
                updated_at = excluded.updated_at
            ",
            params![date, new_increment, timestamp],
        )
        .map_err(|error| format!("failed to update daily stats: {error}"))?;

    transaction
        .commit()
        .map_err(|error| format!("failed to commit review transaction: {error}"))?;

    Ok(())
}

#[tauri::command]
pub fn record_review(app: AppHandle, review: ReviewSubmission) -> Result<(), String> {
    let mut connection = open_database(&app)?;
    record_review_on_connection(&mut connection, review)
}

fn learning_plan_settings_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<LearningPlanSettings> {
    Ok(LearningPlanSettings {
        intensity: row.get(0)?,
        desired_retention: row.get(1)?,
        daily_new_target: row.get(2)?,
        daily_review_limit: row.get(3)?,
        scene_lessons_target: row.get(4)?,
        recovery_days: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn intensity_defaults(intensity: &str) -> (i64, i64, i64) {
    match intensity {
        "light" => (12, 80, 1),
        "intensive" => (50, 300, 5),
        _ => (25, 160, 3),
    }
}

#[tauri::command]
pub fn get_learning_plan_settings(app: AppHandle) -> Result<LearningPlanSettings, String> {
    let connection = open_database(&app)?;
    create_schema(&connection)?;
    connection
        .query_row(
            "
            SELECT intensity, desired_retention, daily_new_target, daily_review_limit,
                scene_lessons_target, recovery_days, updated_at
            FROM learning_plan_settings
            WHERE id = 'default'
            ",
            [],
            learning_plan_settings_from_row,
        )
        .map_err(|error| format!("failed to load learning plan settings: {error}"))
}

#[tauri::command]
pub fn save_learning_plan_settings(
    app: AppHandle,
    request: SaveLearningPlanSettingsRequest,
) -> Result<LearningPlanSettings, String> {
    let connection = open_database(&app)?;
    create_schema(&connection)?;
    let intensity = match request.intensity.as_str() {
        "light" | "intensive" => request.intensity.as_str(),
        _ => "standard",
    };
    let (daily_new_target, daily_review_limit, scene_lessons_target) =
        intensity_defaults(intensity);
    let now = now_string();
    connection
        .execute(
            "
            INSERT INTO learning_plan_settings
                (
                    id, intensity, desired_retention, daily_new_target,
                    daily_review_limit, scene_lessons_target, recovery_days, updated_at
                )
            VALUES
                ('default', ?1, 0.90, ?2, ?3, ?4, 3, ?5)
            ON CONFLICT(id) DO UPDATE SET
                intensity = excluded.intensity,
                daily_new_target = excluded.daily_new_target,
                daily_review_limit = excluded.daily_review_limit,
                scene_lessons_target = excluded.scene_lessons_target,
                updated_at = excluded.updated_at
            ",
            params![
                intensity,
                daily_new_target,
                daily_review_limit,
                scene_lessons_target,
                now
            ],
        )
        .map_err(|error| format!("failed to save learning plan settings: {error}"))?;
    get_learning_plan_settings(app)
}

#[tauri::command]
pub fn get_today_learning_plan(app: AppHandle, plan_date: String) -> Result<DailyLearningPlan, String> {
    let connection = open_database(&app)?;
    create_schema(&connection)?;
    let settings = get_learning_plan_settings(app.clone())?;
    let now = now_string();
    let date_prefix = if plan_date.trim().is_empty() {
        Utc::now().date_naive().to_string()
    } else {
        plan_date.trim().chars().take(10).collect::<String>()
    };
    let due_cutoff = format!("{date_prefix}T23:59:59Z");
    let due_count: i64 = connection
        .query_row(
            "
            SELECT COUNT(*)
            FROM vocabulary_items v
            LEFT JOIN vocabulary_memory_states m
                ON m.vocabulary_item_id = v.id AND m.skill_type = 'recognition'
            WHERE COALESCE(m.due_at, v.next_review_at) IS NOT NULL
                AND COALESCE(m.due_at, v.next_review_at) <= ?1
            ",
            params![due_cutoff],
            |row| row.get(0),
        )
        .map_err(|error| format!("failed to count due words: {error}"))?;
    let weak_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM vocabulary_items WHERE weak_score >= 2",
            [],
            |row| row.get(0),
        )
        .map_err(|error| format!("failed to count weak words: {error}"))?;
    let backlog_count = due_count.saturating_sub(settings.daily_review_limit);
    let new_word_target = if backlog_count > 0 {
        (settings.daily_new_target / 2).max(0)
    } else {
        settings.daily_new_target
    };
    let explanation = if backlog_count > 0 {
        "今天先处理到期词和弱词，新词自动降载，避免复习债滚雪球。".to_string()
    } else {
        "今天按当前强度推进：先背新词，再清理到期词，最后把核心词送进场景课。".to_string()
    };
    let plan_json = serde_json::json!({
        "desiredRetention": settings.desired_retention,
        "recoveryDays": settings.recovery_days,
        "tasks": [
            { "type": "new_words", "target": new_word_target },
            { "type": "reviews", "limit": settings.daily_review_limit, "due": due_count },
            { "type": "scene_lessons", "target": settings.scene_lessons_target },
            { "type": "weak_words", "count": weak_count }
        ]
    })
    .to_string();
    let id = format!("learning_plan_{date_prefix}");
    connection
        .execute(
            "
            INSERT INTO daily_learning_plans
                (
                    id, plan_date, intensity, new_word_target, review_limit,
                    scene_lesson_target, due_count, weak_count, backlog_count,
                    plan_json, explanation, created_at, updated_at
                )
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12)
            ON CONFLICT(plan_date) DO UPDATE SET
                intensity = excluded.intensity,
                new_word_target = excluded.new_word_target,
                review_limit = excluded.review_limit,
                scene_lesson_target = excluded.scene_lesson_target,
                due_count = excluded.due_count,
                weak_count = excluded.weak_count,
                backlog_count = excluded.backlog_count,
                plan_json = excluded.plan_json,
                explanation = excluded.explanation,
                updated_at = excluded.updated_at
            ",
            params![
                id,
                date_prefix,
                settings.intensity,
                new_word_target,
                settings.daily_review_limit,
                settings.scene_lessons_target,
                due_count,
                weak_count,
                backlog_count,
                plan_json,
                explanation,
                now
            ],
        )
        .map_err(|error| format!("failed to save daily learning plan: {error}"))?;
    connection
        .query_row(
            "
            SELECT id, plan_date, intensity, new_word_target, review_limit,
                scene_lesson_target, due_count, weak_count, backlog_count,
                plan_json, explanation, created_at, updated_at
            FROM daily_learning_plans
            WHERE plan_date = ?1
            ",
            params![date_prefix],
            |row| {
                Ok(DailyLearningPlan {
                    id: row.get(0)?,
                    plan_date: row.get(1)?,
                    intensity: row.get(2)?,
                    new_word_target: row.get(3)?,
                    review_limit: row.get(4)?,
                    scene_lesson_target: row.get(5)?,
                    due_count: row.get(6)?,
                    weak_count: row.get(7)?,
                    backlog_count: row.get(8)?,
                    plan_json: row.get(9)?,
                    explanation: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            },
        )
        .map_err(|error| format!("failed to load daily learning plan: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{
        create_schema, delete_course_pack_on_connection, delete_sentence_on_connection,
        merge_duplicate_lessons, normalize_vocabulary_text, parse_vocabulary_book_markdown,
        record_review_on_connection, table_has_column, ReviewSubmission,
    };
    use rusqlite::{params, Connection};

    #[test]
    fn creates_expected_tables() {
        let connection = Connection::open_in_memory().expect("open memory database");
        create_schema(&connection).expect("create schema");

        let mut statement = connection
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .expect("prepare table query");
        let tables = statement
            .query_map([], |row| row.get::<_, String>(0))
            .expect("query tables")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect tables");

        assert!(tables.contains(&"course_packs".to_string()));
        assert!(tables.contains(&"lessons".to_string()));
        assert!(tables.contains(&"sentence_items".to_string()));
        assert!(tables.contains(&"review_logs".to_string()));
        assert!(tables.contains(&"daily_stats".to_string()));
        assert!(tables.contains(&"vocabulary_books".to_string()));
        assert!(tables.contains(&"vocabulary_items".to_string()));
        assert!(tables.contains(&"vocabulary_book_items".to_string()));
        assert!(tables.contains(&"vocabulary_reviews".to_string()));
        assert!(tables.contains(&"sentence_vocabulary_links".to_string()));
        assert!(tables.contains(&"sync_settings".to_string()));
        assert!(tables.contains(&"llm_settings".to_string()));
        assert!(tables.contains(&"generated_scenes".to_string()));
        assert!(tables.contains(&"scene_lines".to_string()));
        assert!(tables.contains(&"scene_vocabulary_links".to_string()));
        assert!(!table_has_column(&connection, "sentence_items", "course_pack_id").unwrap());
        assert!(!table_has_column(&connection, "review_logs", "course_pack_id").unwrap());
    }

    #[test]
    fn parses_markdown_vocabulary_book() {
        let raw = r#"
# CET4 Week 1

## abandon
- 中文释义：放弃；抛弃
- 音标：/əˈbændən/
- 词性：verb
- 例句：She had to abandon the plan.
- 词根词缀：a- 表示离开
- 近义词：give up, quit
- 反义词：keep
- 形象记忆：把计划丢在路边
- 场景标签：study, decision
- 难度：B1

## empty
- 音标：/ˈempti/
"#;

        let (title, entries, skipped) = parse_vocabulary_book_markdown(raw);
        assert_eq!(title, Some("CET4 Week 1".to_string()));
        assert_eq!(entries.len(), 1);
        assert_eq!(skipped, 1);
        assert_eq!(entries[0].text, "abandon");
        assert_eq!(entries[0].primary_meaning, "放弃；抛弃");
        assert_eq!(entries[0].synonyms, "give up, quit");
        assert_eq!(normalize_vocabulary_text("  Abandon!  "), "abandon");
    }

    #[test]
    fn rejects_invalid_status_values() {
        let connection = Connection::open_in_memory().expect("open memory database");
        create_schema(&connection).expect("create schema");

        connection
            .execute(
                "
                INSERT INTO course_packs
                    (id, name, language, daily_new_target, created_at, updated_at)
                VALUES
                    ('course_1', 'Test Course', 'en', 20, '2026-04-26', '2026-04-26')
                ",
                [],
            )
            .expect("insert course");
        connection
            .execute(
                "
                INSERT INTO lessons
                    (id, course_pack_id, title, sort_order, created_at, updated_at)
                VALUES
                    ('lesson_1', 'course_1', 'Lesson 1', 0, '2026-04-26', '2026-04-26')
                ",
                [],
            )
            .expect("insert lesson");

        let result = connection.execute(
            "
            INSERT INTO sentence_items
                (id, lesson_id, english, chinese, status, created_at, updated_at)
            VALUES
                ('sentence_1', 'lesson_1', 'English.', '中文。', 'dirty', '2026-04-26', '2026-04-26')
            ",
            [],
        );

        assert!(result.is_err());
    }

    #[test]
    fn rejects_orphan_sentence_and_review_rows() {
        let connection = Connection::open_in_memory().expect("open memory database");
        create_schema(&connection).expect("create schema");

        let sentence_result = connection.execute(
            "
            INSERT INTO sentence_items
                (id, lesson_id, english, chinese, created_at, updated_at)
            VALUES
                ('sentence_1', 'missing_lesson', 'English.', '中文。', '2026-04-26', '2026-04-26')
            ",
            [],
        );
        assert!(sentence_result.is_err());

        let review_result = connection.execute(
            "
            INSERT INTO review_logs
                (id, sentence_id, user_answer, is_correct, rating, interval_minutes, reviewed_at)
            VALUES
                ('review_1', 'missing_sentence', 'English.', 1, 'good', 10, '2026-04-26')
            ",
            [],
        );
        assert!(review_result.is_err());
    }

    #[test]
    fn merges_duplicate_lessons_before_unique_index() {
        let connection = Connection::open_in_memory().expect("open memory database");
        create_schema(&connection).expect("create schema");
        connection
            .execute("DROP INDEX idx_lessons_course_pack_title", [])
            .expect("drop unique lesson index");

        connection
            .execute(
                "
                INSERT INTO course_packs
                    (id, name, language, daily_new_target, created_at, updated_at)
                VALUES
                    ('course_1', 'Test Course', 'en', 20, '2026-04-26', '2026-04-26')
                ",
                [],
            )
            .expect("insert course");
        connection
            .execute(
                "
                INSERT INTO lessons
                    (id, course_pack_id, title, sort_order, created_at, updated_at)
                VALUES
                    ('lesson_a', 'course_1', 'Lesson 1', 0, '2026-04-26', '2026-04-26'),
                    ('lesson_b', 'course_1', 'Lesson 1', 1, '2026-04-27', '2026-04-27')
                ",
                [],
            )
            .expect("insert duplicate lessons");
        connection
            .execute(
                "
                INSERT INTO sentence_items
                    (id, lesson_id, english, chinese, created_at, updated_at)
                VALUES
                    ('sentence_1', 'lesson_b', 'English.', '中文。', '2026-04-26', '2026-04-26')
                ",
                [],
            )
            .expect("insert sentence");

        merge_duplicate_lessons(&connection).expect("merge duplicate lessons");

        let lesson_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM lessons", [], |row| row.get(0))
            .expect("count lessons");
        assert_eq!(lesson_count, 1);

        let sentence_lesson_id: String = connection
            .query_row(
                "SELECT lesson_id FROM sentence_items WHERE id = 'sentence_1'",
                [],
                |row| row.get(0),
            )
            .expect("query sentence lesson");
        assert_eq!(sentence_lesson_id, "lesson_a");
    }

    fn insert_review_fixture(connection: &Connection) {
        connection
            .execute(
                "
                INSERT INTO course_packs
                    (id, name, language, daily_new_target, created_at, updated_at)
                VALUES
                    ('course_1', 'Test Course', 'en', 20, '2026-04-26', '2026-04-26')
                ",
                [],
            )
            .expect("insert course");
        connection
            .execute(
                "
                INSERT INTO lessons
                    (id, course_pack_id, title, sort_order, created_at, updated_at)
                VALUES
                    ('lesson_1', 'course_1', 'Lesson 1', 0, '2026-04-26', '2026-04-26')
                ",
                [],
            )
            .expect("insert lesson");
        connection
            .execute(
                "
                INSERT INTO sentence_items
                    (id, lesson_id, english, chinese, created_at, updated_at)
                VALUES
                    ('sentence_1', 'lesson_1', 'English.', '中文。', '2026-04-26', '2026-04-26')
                ",
                [],
            )
            .expect("insert sentence");
        connection
            .execute(
                "
                INSERT INTO review_logs
                    (id, sentence_id, user_answer, is_correct, rating, interval_minutes, reviewed_at)
                VALUES
                    ('review_1', 'sentence_1', 'English.', 1, 'good', 10, '2026-04-26')
                ",
                [],
            )
            .expect("insert review");
    }

    #[test]
    fn delete_sentence_cascades_reviews_only() {
        let connection = Connection::open_in_memory().expect("open memory database");
        create_schema(&connection).expect("create schema");
        insert_review_fixture(&connection);

        delete_sentence_on_connection(&connection, "sentence_1").expect("delete sentence");

        let sentence_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM sentence_items", [], |row| row.get(0))
            .expect("count sentences");
        let review_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM review_logs", [], |row| row.get(0))
            .expect("count reviews");
        let lesson_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM lessons", [], |row| row.get(0))
            .expect("count lessons");
        let course_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM course_packs", [], |row| row.get(0))
            .expect("count courses");

        assert_eq!(sentence_count, 0);
        assert_eq!(review_count, 0);
        assert_eq!(lesson_count, 1);
        assert_eq!(course_count, 1);
    }

    #[test]
    fn delete_course_pack_cascades_lessons_sentences_and_reviews() {
        let connection = Connection::open_in_memory().expect("open memory database");
        create_schema(&connection).expect("create schema");
        insert_review_fixture(&connection);

        delete_course_pack_on_connection(&connection, "course_1").expect("delete course");

        let course_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM course_packs", [], |row| row.get(0))
            .expect("count courses");
        let lesson_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM lessons", [], |row| row.get(0))
            .expect("count lessons");
        let sentence_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM sentence_items", [], |row| row.get(0))
            .expect("count sentences");
        let review_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM review_logs", [], |row| row.get(0))
            .expect("count reviews");

        assert_eq!(course_count, 0);
        assert_eq!(lesson_count, 0);
        assert_eq!(sentence_count, 0);
        assert_eq!(review_count, 0);
    }

    #[test]
    fn record_review_updates_sentence_log_and_daily_stats() {
        let mut connection = Connection::open_in_memory().expect("open memory database");
        create_schema(&connection).expect("create schema");

        connection
            .execute(
                "
                INSERT INTO course_packs
                    (id, name, language, daily_new_target, created_at, updated_at)
                VALUES
                    ('course_1', 'Test Course', 'en', 20, '2026-04-26', '2026-04-26')
                ",
                [],
            )
            .expect("insert course");
        connection
            .execute(
                "
                INSERT INTO lessons
                    (id, course_pack_id, title, sort_order, created_at, updated_at)
                VALUES
                    ('lesson_1', 'course_1', 'Lesson 1', 0, '2026-04-26', '2026-04-26')
                ",
                [],
            )
            .expect("insert lesson");
        connection
            .execute(
                "
                INSERT INTO sentence_items
                    (
                        id,
                        lesson_id,
                        english,
                        chinese,
                        phonetic,
                        note,
                        created_at,
                        updated_at
                    )
                VALUES
                    (
                        'sentence_1',
                        'lesson_1',
                        'The market is strong.',
                        '市场很强劲。',
                        '',
                        '',
                        '2026-04-26',
                        '2026-04-26'
                    )
                ",
                [],
            )
            .expect("insert sentence");

        record_review_on_connection(
            &mut connection,
            ReviewSubmission {
                id: "review_1".to_string(),
                sentence_id: "sentence_1".to_string(),
                user_answer: "The market is weak.".to_string(),
                is_correct: false,
                wrong_indexes: vec![3],
                rating: "again".to_string(),
                reviewed_at: "2026-04-26T10:00:00".to_string(),
                interval_minutes: 10,
                next_review_at: Some("2026-04-26T10:10:00".to_string()),
            },
        )
        .expect("record review");

        let sentence_state: (String, i64, i64, i64, Option<String>) = connection
            .query_row(
                "
                SELECT status, show_count, review_count, error_count, next_review_at
                FROM sentence_items
                WHERE id = 'sentence_1'
                ",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .expect("query sentence");
        assert_eq!(sentence_state.0, "learning");
        assert_eq!(sentence_state.1, 1);
        assert_eq!(sentence_state.2, 1);
        assert_eq!(sentence_state.3, 1);
        assert_eq!(sentence_state.4, Some("2026-04-26T10:10:00".to_string()));

        let log_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM review_logs", [], |row| row.get(0))
            .expect("query review count");
        assert_eq!(log_count, 1);

        let daily_stats: (i64, i64) = connection
            .query_row(
                "
                SELECT new_count, review_count
                FROM daily_stats
                WHERE date = ?1
                ",
                params!["2026-04-26"],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("query daily stats");
        assert_eq!(daily_stats, (1, 1));
    }
}
