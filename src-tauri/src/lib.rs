mod commands;
mod state;
mod template;
mod tray;
mod widget_manager;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .manage(std::sync::Mutex::new(state::AppState::default()))
        .setup(|app| {
            tray::create_tray(app)?;

            let handle = app.handle().clone();
            let saved_widgets = commands::config::load_widgets(&handle);

            if !saved_widgets.is_empty() {
                let app_state = app.state::<std::sync::Mutex<state::AppState>>();
                let mut locked = app_state.lock().unwrap();
                for w in &saved_widgets {
                    let html_path = template::get_template_html_path(&w.template_id);
                    if let Some(path) = html_path {
                        if let Err(e) = widget_manager::create_widget_window(
                            app.handle(),
                            &w.label,
                            &path,
                            w.size.0,
                            w.size.1,
                        ) {
                            eprintln!("Failed to restore widget '{}': {e}", w.label);
                            continue;
                        }
                        locked.add_widget(w.clone());
                    }
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::widget::create_widget,
            commands::widget::destroy_widget,
            commands::widget::list_widgets,
            commands::widget::show_widget,
            commands::widget::hide_widget,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
