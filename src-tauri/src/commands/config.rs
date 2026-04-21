use crate::state::WidgetInfo;
use tauri_plugin_store::StoreExt;

const STORE_ID: &str = "widgets.json";
const WIDGETS_KEY: &str = "widgets";

pub fn save_widgets(app: &tauri::AppHandle, widgets: &[WidgetInfo]) {
    let store = match app.store(STORE_ID) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to open store: {e}");
            return;
        }
    };

    let json = serde_json::to_value(widgets).unwrap_or_default();
    store.set(WIDGETS_KEY, json);

    if let Err(e) = store.save() {
        eprintln!("Failed to save widgets: {e}");
    }
}

pub fn load_widgets(app: &tauri::AppHandle) -> Vec<WidgetInfo> {
    let store = match app.store(STORE_ID) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to open store for loading: {e}");
            return vec![];
        }
    };

    match store.get(WIDGETS_KEY) {
        Some(value) => serde_json::from_value::<Vec<WidgetInfo>>(value).unwrap_or_default(),
        None => vec![],
    }
}

#[cfg(test)]
mod tests {
    use crate::state::WidgetInfo;

    fn sample_widget(label: &str) -> WidgetInfo {
        WidgetInfo {
            label: label.to_string(),
            template_id: "clock".to_string(),
            title: "Test".to_string(),
            position: (0.0, 0.0),
            size: (200.0, 80.0),
            opacity: 1.0,
            always_on_top: true,
            click_through: false,
            visible: true,
        }
    }

    #[test]
    fn test_widget_info_serialization() {
        let widgets = vec![sample_widget("w1"), sample_widget("w2")];
        let json = serde_json::to_value(&widgets).unwrap();
        let deserialized: Vec<WidgetInfo> = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.len(), 2);
        assert_eq!(deserialized[0].label, "w1");
        assert_eq!(deserialized[1].label, "w2");
    }

    #[test]
    fn test_empty_widgets_serialization() {
        let widgets: Vec<WidgetInfo> = vec![];
        let json = serde_json::to_value(&widgets).unwrap();
        let deserialized: Vec<WidgetInfo> = serde_json::from_value(json).unwrap();
        assert!(deserialized.is_empty());
    }
}
