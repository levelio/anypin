use crate::commands::config;
use crate::state::AppState;
use crate::template;
use crate::widget_manager;
use tauri::{Manager, State, WebviewUrl, WebviewWindowBuilder};

const URL_INPUT_LABEL: &str = "url-input-dialog";

#[tauri::command]
pub fn create_widget(
    app: tauri::AppHandle,
    state: State<'_, std::sync::Mutex<AppState>>,
    template_id: String,
) -> Result<String, String> {
    let tmpl = template::get_builtin_templates()
        .into_iter()
        .find(|t| t.id == template_id)
        .ok_or_else(|| format!("Template '{template_id}' not found"))?;

    let html_path = template::get_template_html_path(&template_id)
        .ok_or_else(|| format!("No HTML path for template '{template_id}'"))?;

    let label = format!(
        "{}-{}",
        template_id,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    widget_manager::create_widget_window(
        &app,
        &label,
        &html_path,
        tmpl.default_size.0,
        tmpl.default_size.1,
    )?;

    let widget = crate::state::WidgetInfo {
        label: label.clone(),
        template_id: template_id.clone(),
        title: tmpl.name.clone(),
        position: (0.0, 0.0),
        size: tmpl.default_size,
        opacity: 1.0,
        always_on_top: true,
        click_through: false,
        visible: true,
        url: None,
    };

    state.lock().unwrap().add_widget(widget);
    let widgets = state.lock().unwrap().widgets.clone();
    config::save_widgets(&app, &widgets);
    crate::tray::rebuild_tray_menu(&app).map_err(|e| e.to_string())?;

    Ok(label)
}

#[tauri::command]
pub fn create_url_widget(
    app: tauri::AppHandle,
    state: State<'_, std::sync::Mutex<AppState>>,
    url: String,
    title: Option<String>,
    width: Option<f64>,
    height: Option<f64>,
) -> Result<String, String> {
    let w = width.unwrap_or(400.0);
    let h = height.unwrap_or(300.0);
    let display_title = title.unwrap_or_else(|| "Web Pin".to_string());

    let label = format!(
        "web-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    widget_manager::create_url_widget_window(&app, &label, &url, w, h)?;

    let widget = crate::state::WidgetInfo {
        label: label.clone(),
        template_id: "web".to_string(),
        title: display_title,
        position: (0.0, 0.0),
        size: (w, h),
        opacity: 1.0,
        always_on_top: true,
        click_through: false,
        visible: true,
        url: Some(url),
    };

    state.lock().unwrap().add_widget(widget);
    let widgets = state.lock().unwrap().widgets.clone();
    config::save_widgets(&app, &widgets);
    crate::tray::rebuild_tray_menu(&app).map_err(|e| e.to_string())?;

    Ok(label)
}

#[tauri::command]
pub fn destroy_widget(
    app: tauri::AppHandle,
    state: State<'_, std::sync::Mutex<AppState>>,
    label: String,
) -> Result<(), String> {
    widget_manager::destroy_widget_window(&app, &label)?;
    state.lock().unwrap().remove_widget(&label);
    let widgets = state.lock().unwrap().widgets.clone();
    config::save_widgets(&app, &widgets);
    crate::tray::rebuild_tray_menu(&app).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_widgets(state: State<'_, std::sync::Mutex<AppState>>) -> Vec<crate::state::WidgetInfo> {
    state.lock().unwrap().widgets.clone()
}

#[tauri::command]
pub fn show_widget(app: tauri::AppHandle, label: String) -> Result<(), String> {
    let win = app
        .get_webview_window(&label)
        .ok_or_else(|| format!("Window '{label}' not found"))?;
    win.show().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn hide_widget(app: tauri::AppHandle, label: String) -> Result<(), String> {
    let win = app
        .get_webview_window(&label)
        .ok_or_else(|| format!("Window '{label}' not found"))?;
    win.hide().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_widget_opacity(
    app: tauri::AppHandle,
    label: String,
    opacity: f32,
) -> Result<(), String> {
    widget_manager::set_widget_opacity(&app, &label, opacity)
}

#[tauri::command]
pub fn toggle_click_through(
    app: tauri::AppHandle,
    state: State<'_, std::sync::Mutex<AppState>>,
    label: String,
    enabled: bool,
) -> Result<(), String> {
    widget_manager::set_click_through(&app, &label, enabled)?;
    if let Some(w) = state.lock().unwrap().get_widget_mut(&label) {
        w.click_through = enabled;
    }
    let widgets = state.lock().unwrap().widgets.clone();
    config::save_widgets(&app, &widgets);
    crate::tray::rebuild_tray_menu(&app).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn show_widget_context_menu(app: tauri::AppHandle, label: String) -> Result<(), String> {
    widget_manager::popup_widget_menu(&app, &label)
}

#[tauri::command]
pub fn open_url_input(app: tauri::AppHandle) -> Result<(), String> {
    if app.get_webview_window(URL_INPUT_LABEL).is_some() {
        if let Some(win) = app.get_webview_window(URL_INPUT_LABEL) {
            let _ = win.show();
            let _ = win.set_focus();
        }
        return Ok(());
    }
    let url = WebviewUrl::App("widgets/url-input/index.html".into());
    WebviewWindowBuilder::new(&app, URL_INPUT_LABEL, url)
        .title("从 URL 创建 Pin")
        .inner_size(400.0, 280.0)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn close_url_input(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(URL_INPUT_LABEL) {
        win.destroy().map_err(|e| e.to_string())?;
    }
    Ok(())
}
