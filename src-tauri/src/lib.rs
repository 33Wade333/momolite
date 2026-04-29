mod db;
mod llm;
mod sync;
mod tts;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_stronghold::Builder::new(|password| {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(b"momolite-stronghold-v1");
                hasher.update(password.as_bytes());
                hasher.finalize().to_vec()
            })
            .build(),
        )
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
            db::import_vocabulary_book,
            db::list_vocabulary_books,
            db::list_vocabulary_items,
            db::get_vocabulary_book,
            db::get_today_vocabulary_queue,
            db::review_vocabulary_item,
            db::get_learning_plan_settings,
            db::save_learning_plan_settings,
            db::get_today_learning_plan,
            sync::get_secret_vault_key,
            sync::get_sync_settings,
            sync::save_sync_settings,
            sync::test_webdav_connection,
            sync::run_webdav_sync,
            llm::get_llm_settings,
            llm::save_llm_settings,
            llm::list_llm_profiles,
            llm::save_llm_profile,
            llm::set_default_llm_profile,
            llm::delete_llm_profile,
            llm::test_llm_profile,
            llm::enrich_vocabulary_words,
            llm::list_generated_scenes,
            llm::get_generated_scene,
            llm::plan_scene_batch,
            llm::get_scene_batch_plan,
            llm::generate_scene_batch,
            llm::add_scenes_to_course,
            llm::generate_scene_from_vocabulary,
            db::record_review,
            tts::synthesize_piper_tts
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
