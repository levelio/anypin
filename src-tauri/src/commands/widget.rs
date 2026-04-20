use crate::commands::config;
use crate::state::AppState;
use crate::template;
use crate::widget_manager;
use tauri::{Manager, State};

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
    };

    state.lock().unwrap().add_widget(widget);
    let widgets = state.lock().unwrap().widgets.clone();
    config::save_widgets(&app, &widgets);

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
