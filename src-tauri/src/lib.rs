mod db;
mod tts;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            db::init_database,
            db::load_app_state,
            db::list_course_packs,
            db::create_course_pack,
            db::delete_course_pack,
            db::list_lessons,
            db::create_lesson,
            db::list_sentences,
            db::create_sentence,
            db::delete_sentence,
            db::record_review,
            tts::synthesize_piper_tts
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
