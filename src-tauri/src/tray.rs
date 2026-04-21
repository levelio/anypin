use crate::template;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::TrayIconBuilder,
    Manager,
};

const TRAY_ID: &str = "main";

pub fn create_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_menu(app)?;

    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("anyPin - 置顶小部件")
        .on_menu_event(move |app, event| match event.id().as_ref() {
            id if id.starts_with("create-") => {
                let template_id = &id[7..];
                let label = format!(
                    "{}-{}",
                    template_id,
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                );
                if let Err(e) = invoke_create_widget(app, &label, template_id) {
                    eprintln!("Failed to create widget: {e}");
                }
                if let Err(e) = rebuild_tray_menu(app) {
                    eprintln!("Failed to refresh tray menu: {e}");
                }
            }
            id if id.starts_with("focus-") => {
                let lbl = &id[6..];
                if let Some(win) = app.get_webview_window(lbl) {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
            id if id.starts_with("toggle-ct-") => {
                let lbl = &id[10..];
                let current = {
                    let state = app.state::<std::sync::Mutex<crate::state::AppState>>();
                    let val = state
                        .lock()
                        .unwrap()
                        .get_widget(lbl)
                        .map(|w| w.click_through)
                        .unwrap_or(false);
                    val
                };
                let new_val = !current;
                if crate::widget_manager::set_click_through(app, lbl, new_val).is_ok() {
                    {
                        let state = app.state::<std::sync::Mutex<crate::state::AppState>>();
                        let mut locked = state.lock().unwrap();
                        if let Some(w) = locked.get_widget_mut(lbl) {
                            w.click_through = new_val;
                        }
                        let widgets = locked.widgets.clone();
                        crate::commands::config::save_widgets(app, &widgets);
                    }
                    if let Err(e) = rebuild_tray_menu(app) {
                        eprintln!("Failed to refresh tray menu: {e}");
                    }
                }
            }
            "quit" => {
                app.exit(0);
            }
            "cancel-all-ct" => {
                let ct_labels: Vec<String> = {
                    let state = app.state::<std::sync::Mutex<crate::state::AppState>>();
                    let labels = state
                        .lock()
                        .unwrap()
                        .widgets
                        .iter()
                        .filter(|w| w.click_through)
                        .map(|w| w.label.clone())
                        .collect::<Vec<_>>();
                    labels
                };
                for lbl in &ct_labels {
                    {
                        let state = app.state::<std::sync::Mutex<crate::state::AppState>>();
                        let mut locked = state.lock().unwrap();
                        if let Some(w) = locked.get_widget_mut(lbl) {
                            w.click_through = false;
                        }
                    }
                    let _ = crate::widget_manager::set_click_through(app, lbl, false);
                }
                {
                    let state = app.state::<std::sync::Mutex<crate::state::AppState>>();
                    let widgets = state.lock().unwrap().widgets.clone();
                    crate::commands::config::save_widgets(app, &widgets);
                }
                if let Err(e) = rebuild_tray_menu(app) {
                    eprintln!("Failed to refresh tray menu: {e}");
                }
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}

fn build_menu<M: Manager<tauri::Wry>>(
    app: &M,
) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let templates = template::get_builtin_templates();
    let create_items: Vec<MenuItem<tauri::Wry>> = templates
        .iter()
        .map(|t| {
            MenuItem::with_id(
                app,
                format!("create-{}", t.id),
                format!("新建{}", t.name),
                true,
                None::<&str>,
            )
            .unwrap()
        })
        .collect();
    let create_refs: Vec<&dyn tauri::menu::IsMenuItem<_>> = create_items
        .iter()
        .map(|i| i as &dyn tauri::menu::IsMenuItem<_>)
        .collect();
    let create_menu = Submenu::with_items(app, "新建 Pin", true, &create_refs)?;

    let sep1 = PredefinedMenuItem::separator(app)?;

    let widgets = {
        let state = app.state::<std::sync::Mutex<crate::state::AppState>>();
        let locked = state.lock().unwrap();
        locked.widgets.clone()
    };

    let active_refs: Vec<Box<dyn tauri::menu::IsMenuItem<tauri::Wry>>> = if widgets.is_empty() {
        vec![Box::new(MenuItem::with_id(
            app,
            "no-widgets",
            "暂无置顶窗口",
            false,
            None::<&str>,
        )?)]
    } else {
        widgets
            .iter()
            .map(|w| {
                let focus_item = MenuItem::with_id(
                    app,
                    format!("focus-{}", w.label),
                    "聚焦窗口",
                    true,
                    None::<&str>,
                )
                .unwrap();
                let ct_label = if w.click_through { "取消穿透" } else { "开启穿透" };
                let ct_item = MenuItem::with_id(
                    app,
                    format!("toggle-ct-{}", w.label),
                    ct_label,
                    true,
                    None::<&str>,
                )
                .unwrap();
                let items: Vec<&dyn tauri::menu::IsMenuItem<_>> =
                    vec![&focus_item, &ct_item];
                let title = if w.click_through {
                    format!("{} [穿透]", w.title)
                } else {
                    w.title.clone()
                };
                Box::new(Submenu::with_items(app, title, true, &items).unwrap())
                    as Box<dyn tauri::menu::IsMenuItem<_>>
            })
            .collect()
    };

    let active_refs: Vec<&dyn tauri::menu::IsMenuItem<_>> =
        active_refs.iter().map(|i| i.as_ref()).collect();
    let active_i = Submenu::with_items(app, "活跃 Pins", true, &active_refs)?;

    let sep2 = PredefinedMenuItem::separator(app)?;

    let cancel_ct = widgets.iter().any(|w| w.click_through);
    let cancel_ct_i = MenuItem::with_id(
        app,
        "cancel-all-ct",
        "取消全部穿透",
        cancel_ct,
        None::<&str>,
    )?;

    let sep3 = PredefinedMenuItem::separator(app)?;
    let quit_i = MenuItem::with_id(app, "quit", "退出 anyPin", true, None::<&str>)?;

    Ok(Menu::with_items(
        app,
        &[&create_menu, &sep1, &active_i, &sep2, &cancel_ct_i, &sep3, &quit_i],
    )?)
}

pub fn rebuild_tray_menu(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_menu(app)?;
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}

fn invoke_create_widget(
    app: &tauri::AppHandle,
    label: &str,
    template_id: &str,
) -> Result<(), String> {
    let tmpl = template::get_builtin_templates()
        .into_iter()
        .find(|t| t.id == template_id)
        .ok_or_else(|| format!("Template '{template_id}' not found"))?;

    let html_path = template::get_template_html_path(template_id)
        .ok_or_else(|| format!("No HTML for '{template_id}'"))?;

    crate::widget_manager::create_widget_window(
        app,
        label,
        &html_path,
        tmpl.default_size.0,
        tmpl.default_size.1,
    )?;

    let widget = crate::state::WidgetInfo {
        label: label.to_string(),
        template_id: template_id.to_string(),
        title: tmpl.name.clone(),
        position: (0.0, 0.0),
        size: tmpl.default_size,
        opacity: 1.0,
        always_on_top: true,
        click_through: false,
        visible: true,
    };

    let state = app.state::<std::sync::Mutex<crate::state::AppState>>();
    state.lock().unwrap().add_widget(widget);
    let widgets = state.lock().unwrap().widgets.clone();
    crate::commands::config::save_widgets(app, &widgets);

    Ok(())
}
