use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

pub fn create_widget_window(
    app: &tauri::AppHandle,
    label: &str,
    html_path: &str,
    width: f64,
    height: f64,
) -> Result<(), String> {
    let url = WebviewUrl::App(html_path.into());

    let mut builder = WebviewWindowBuilder::new(app, label, url)
        .always_on_top(true)
        .decorations(false)
        .skip_taskbar(true)
        .transparent(true)
        .inner_size(width, height);

    #[cfg(target_os = "macos")]
    {
        builder = builder.visible_on_all_workspaces(true);
    }

    builder.build().map_err(|e| e.to_string())?;

    Ok(())
}

pub fn destroy_widget_window(app: &tauri::AppHandle, label: &str) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(label) {
        win.destroy().map_err(|e| e.to_string())
    } else {
        Err(format!("Window '{label}' not found"))
    }
}
