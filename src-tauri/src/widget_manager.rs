use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

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
