use rusqlite::{params, Connection};
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

fn now_string() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    seconds.to_string()
}

fn database_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("failed to resolve app data dir: {error}"))?;

    fs::create_dir_all(&app_dir)
        .map_err(|error| format!("failed to create app data dir: {error}"))?;

    Ok(app_dir.join(DB_FILE_NAME))
}

fn open_database(app: &AppHandle) -> Result<Connection, String> {
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

fn create_schema(connection: &Connection) -> Result<(), String> {
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
            ",
        )
        .map_err(|error| format!("failed to create database schema: {error}"))?;

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
pub fn list_sentences(
    app: AppHandle,
    course_pack_id: String,
) -> Result<Vec<SentenceItem>, String> {
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

#[cfg(test)]
mod tests {
    use super::{
        create_schema, delete_course_pack_on_connection, delete_sentence_on_connection,
        merge_duplicate_lessons, record_review_on_connection, table_has_column, ReviewSubmission,
    };
    use rusqlite::{params, Connection};

    #[test]
    fn creates_expected_tables() {
        let connection = Connection::open_in_memory().expect("open memory database");
        create_schema(&connection).expect("create schema");

        let mut statement = connection
            .prepare(
                "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name",
            )
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
        assert!(!table_has_column(&connection, "sentence_items", "course_pack_id").unwrap());
        assert!(!table_has_column(&connection, "review_logs", "course_pack_id").unwrap());
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
        assert_eq!(
            sentence_state.4,
            Some("2026-04-26T10:10:00".to_string())
        );

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
