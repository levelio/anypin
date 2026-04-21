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
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    use tauri_plugin_global_shortcut::ShortcutState;
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    let id = shortcut.to_string();
                    match id.as_str() {
                        "CmdOrCtrl+Shift+H" => {
                            toggle_all_widgets(app);
                        }
                        "CmdOrCtrl+Shift+N" => {
                            let _ = commands::widget::open_url_input(app.clone());
                        }
                        _ => {}
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .manage(std::sync::Mutex::new(state::AppState::default()))
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            tray::create_tray(app)?;

            let handle = app.handle().clone();
            let saved_widgets = commands::config::load_widgets(&handle);

            if !saved_widgets.is_empty() {
                let app_state = app.state::<std::sync::Mutex<state::AppState>>();
                let mut locked = app_state.lock().unwrap();
                for w in &saved_widgets {
                    let result = if let Some(ref url) = w.url {
                        widget_manager::create_url_widget_window(
                            app.handle(),
                            &w.label,
                            url,
                            w.size.0,
                            w.size.1,
                        )
                    } else {
                        let html_path = template::get_template_html_path(&w.template_id);
                        match html_path {
                            Some(path) => widget_manager::create_widget_window(
                                app.handle(),
                                &w.label,
                                &path,
                                w.size.0,
                                w.size.1,
                            ),
                            None => {
                                eprintln!(
                                    "Failed to restore widget '{}': no HTML path",
                                    w.label
                                );
                                continue;
                            }
                        }
                    };
                    if let Err(e) = result {
                        eprintln!("Failed to restore widget '{}': {e}", w.label);
                        continue;
                    }
                    locked.add_widget(w.clone());
                }
                drop(locked);
                tray::rebuild_tray_menu(app.handle())?;
            }

            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::GlobalShortcutExt;
                app.global_shortcut()
                    .register("CmdOrCtrl+Shift+H")?;
                app.global_shortcut()
                    .register("CmdOrCtrl+Shift+N")?;
            }

            Ok(())
        })
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            if !id.starts_with("ctx:") {
                return;
            }
            let rest = &id[4..];
            let colon_pos = match rest.rfind(':') {
                Some(p) => p,
                None => return,
            };
            let label = &rest[..colon_pos];
            let action = &rest[colon_pos + 1..];

            match action {
                "op100" => {
                    let _ = widget_manager::set_widget_opacity(app, label, 1.0);
                }
                "op70" => {
                    let _ = widget_manager::set_widget_opacity(app, label, 0.7);
                }
                "op50" => {
                    let _ = widget_manager::set_widget_opacity(app, label, 0.5);
                }
                "op30" => {
                    let _ = widget_manager::set_widget_opacity(app, label, 0.3);
                }
                "ct" => {
                    let current = {
                        let state = app.state::<std::sync::Mutex<state::AppState>>();
                        let val = state
                            .lock()
                            .unwrap()
                            .get_widget(label)
                            .map(|w| w.click_through)
                            .unwrap_or(false);
                        val
                    };
                    let new_val = !current;
                    if widget_manager::set_click_through(app, label, new_val).is_ok() {
                        {
                            let state = app.state::<std::sync::Mutex<state::AppState>>();
                            let mut locked = state.lock().unwrap();
                            if let Some(w) = locked.get_widget_mut(label) {
                                w.click_through = new_val;
                            }
                            let widgets = locked.widgets.clone();
                            commands::config::save_widgets(app, &widgets);
                        }
                        let _ = tray::rebuild_tray_menu(app);
                    }
                }
                "close" => {
                    if widget_manager::destroy_widget_window(app, label).is_ok() {
                        {
                            let state = app.state::<std::sync::Mutex<state::AppState>>();
                            state.lock().unwrap().remove_widget(label);
                            let widgets = state.lock().unwrap().widgets.clone();
                            commands::config::save_widgets(app, &widgets);
                        }
                        let _ = tray::rebuild_tray_menu(app);
                    }
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::widget::create_widget,
            commands::widget::destroy_widget,
            commands::widget::list_widgets,
            commands::widget::show_widget,
            commands::widget::hide_widget,
            commands::widget::set_widget_opacity,
            commands::widget::toggle_click_through,
            commands::widget::show_widget_context_menu,
            commands::widget::create_url_widget,
            commands::widget::open_url_input,
            commands::widget::close_url_input,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn toggle_all_widgets(app: &tauri::AppHandle) {
    use tauri::Manager;
    let state = app.state::<std::sync::Mutex<state::AppState>>();
    let any_visible = state
        .lock()
        .unwrap()
        .widgets
        .iter()
        .any(|w| w.visible);
    let show = !any_visible;

    let labels: Vec<(String, bool)> = state
        .lock()
        .unwrap()
        .widgets
        .iter()
        .map(|w| (w.label.clone(), show))
        .collect();

    for (label, vis) in &labels {
        if *vis {
            if let Some(win) = app.get_webview_window(label) {
                let _ = win.show();
            }
        } else if let Some(win) = app.get_webview_window(label) {
            let _ = win.hide();
        }
    }

    {
        let mut locked = state.lock().unwrap();
        for w in &mut locked.widgets {
            w.visible = show;
        }
        let widgets = locked.widgets.clone();
        commands::config::save_widgets(app, &widgets);
    }
}
