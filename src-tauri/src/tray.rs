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
            "create-clock" => {
                let label = format!(
                    "clock-{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                );
                if let Err(e) = invoke_create_clock(app, &label) {
                    eprintln!("Failed to create clock widget: {e}");
                }
                if let Err(e) = rebuild_tray_menu(app) {
                    eprintln!("Failed to refresh tray menu: {e}");
                }
            }
            id if id.starts_with("focus-") => {
                let label = &id[6..];
                if let Some(win) = app.get_webview_window(label) {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}

fn build_menu<M: Manager<tauri::Wry>>(
    app: &M,
) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let clock_i = MenuItem::with_id(app, "create-clock", "新建置顶时钟", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;

    let widgets = {
        let state = app.state::<std::sync::Mutex<crate::state::AppState>>();
        let locked = state.lock().unwrap();
        locked.widgets.clone()
    };

    let active_items: Vec<MenuItem<tauri::Wry>> = if widgets.is_empty() {
        vec![MenuItem::with_id(
            app,
            "no-widgets",
            "暂无置顶窗口",
            false,
            None::<&str>,
        )?]
    } else {
        widgets
            .iter()
            .map(|w| {
                let focus_id = format!("focus-{}", w.label);
                MenuItem::with_id(app, &focus_id, &w.title, true, None::<&str>).unwrap()
            })
            .collect()
    };

    let active_refs: Vec<&dyn tauri::menu::IsMenuItem<_>> = active_items
        .iter()
        .map(|i| i as &dyn tauri::menu::IsMenuItem<_>)
        .collect();
    let active_i = Submenu::with_items(app, "活跃 Pins", true, &active_refs)?;

    let sep2 = PredefinedMenuItem::separator(app)?;
    let quit_i = MenuItem::with_id(app, "quit", "退出 anyPin", true, None::<&str>)?;

    Ok(Menu::with_items(
        app,
        &[&clock_i, &sep1, &active_i, &sep2, &quit_i],
    )?)
}

pub fn rebuild_tray_menu(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_menu(app)?;
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}

fn invoke_create_clock(app: &tauri::AppHandle, label: &str) -> Result<(), String> {
    crate::widget_manager::create_widget_window(
        app,
        label,
        "widgets/clock/index.html",
        200.0,
        80.0,
    )?;

    let widget = crate::state::WidgetInfo {
        label: label.to_string(),
        template_id: "clock".to_string(),
        title: "置顶时钟".to_string(),
        position: (0.0, 0.0),
        size: (200.0, 80.0),
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
