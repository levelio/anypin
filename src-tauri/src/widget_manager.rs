use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    Manager, WebviewUrl, WebviewWindowBuilder,
};

pub fn create_widget_window(
    app: &tauri::AppHandle,
    label: &str,
    html_path: &str,
    width: f64,
    height: f64,
) -> Result<(), String> {
    let url_with_label = format!("{html_path}?label={label}");
    let url = WebviewUrl::App(url_with_label.into());

    let webview = WebviewWindowBuilder::new(app, label, url)
        .always_on_top(true)
        .decorations(false)
        .skip_taskbar(true)
        .transparent(true)
        .inner_size(width, height)
        .build()
        .map_err(|e| e.to_string())?;

    #[cfg(target_os = "macos")]
    {
        webview
            .set_visible_on_all_workspaces(true)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn create_url_widget_window(
    app: &tauri::AppHandle,
    label: &str,
    url: &str,
    width: f64,
    height: f64,
) -> Result<(), String> {
    let webview_url = WebviewUrl::External(url.parse().map_err(|e: url::ParseError| e.to_string())?);

    let webview = WebviewWindowBuilder::new(app, label, webview_url)
        .always_on_top(true)
        .decorations(false)
        .skip_taskbar(true)
        .transparent(true)
        .inner_size(width, height)
        .build()
        .map_err(|e| e.to_string())?;

    #[cfg(target_os = "macos")]
    {
        webview
            .set_visible_on_all_workspaces(true)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn destroy_widget_window(app: &tauri::AppHandle, label: &str) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(label) {
        win.destroy().map_err(|e| e.to_string())
    } else {
        Err(format!("Window '{label}' not found"))
    }
}

pub fn set_widget_opacity(app: &tauri::AppHandle, label: &str, opacity: f32) -> Result<(), String> {
    let win = app
        .get_webview_window(label)
        .ok_or_else(|| format!("Window '{label}' not found"))?;

    #[cfg(target_os = "macos")]
    {
        #[allow(deprecated)]
        {
            use cocoa::appkit::NSWindow;
            use cocoa::base::id;
            let ns_window: id = win.ns_window().map_err(|e| e.to_string())? as id;
            unsafe {
                ns_window.setAlphaValue_(opacity as f64);
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (win, opacity);
    }

    Ok(())
}

pub fn set_click_through(app: &tauri::AppHandle, label: &str, enabled: bool) -> Result<(), String> {
    let win = app
        .get_webview_window(label)
        .ok_or_else(|| format!("Window '{label}' not found"))?;
    win.set_ignore_cursor_events(enabled)
        .map_err(|e| e.to_string())
}

pub fn popup_widget_menu(app: &tauri::AppHandle, label: &str) -> Result<(), String> {
    let win = app
        .get_webview_window(label)
        .ok_or_else(|| format!("Window '{label}' not found"))?;

    let op100 = MenuItem::with_id(app, format!("ctx:{label}:op100"), "100%", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let op70 = MenuItem::with_id(app, format!("ctx:{label}:op70"), "70%", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let op50 = MenuItem::with_id(app, format!("ctx:{label}:op50"), "50%", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let op30 = MenuItem::with_id(app, format!("ctx:{label}:op30"), "30%", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let sep1 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let ct = MenuItem::with_id(app, format!("ctx:{label}:ct"), "点击穿透", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let sep2 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let close = MenuItem::with_id(app, format!("ctx:{label}:close"), "关闭", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let menu = Menu::with_items(
        app,
        &[&op100, &op70, &op50, &op30, &sep1, &ct, &sep2, &close],
    )
    .map_err(|e| e.to_string())?;

    win.popup_menu(&menu).map_err(|e| e.to_string())
}
