mod store;
mod constants;
mod validation;
mod steam;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle();
            if let Err(e) = tauri::async_runtime::block_on(crate::store::initialize_store(&handle)) {
                eprintln!("Failed to initialize store: {}", e);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            crate::store::store_get_key,
            crate::store::store_set_key,
            crate::validation::validate_paths,
			crate::steam::list_steam_game_paths,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
