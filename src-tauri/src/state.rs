use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetInfo {
    pub label: String,
    pub template_id: String,
    pub title: String,
    pub position: (f64, f64),
    pub size: (f64, f64),
    pub opacity: f32,
    pub always_on_top: bool,
    pub click_through: bool,
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub default_size: (f64, f64),
}

#[derive(Debug, Default)]
pub struct AppState {
    pub widgets: Vec<WidgetInfo>,
    #[allow(dead_code)]
    pub templates: Vec<TemplateInfo>,
}

impl AppState {
    pub fn add_widget(&mut self, widget: WidgetInfo) {
        self.widgets.push(widget);
    }

    pub fn remove_widget(&mut self, label: &str) -> Option<WidgetInfo> {
        if let Some(pos) = self.widgets.iter().position(|w| w.label == label) {
            Some(self.widgets.remove(pos))
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn get_widget(&self, label: &str) -> Option<&WidgetInfo> {
        self.widgets.iter().find(|w| w.label == label)
    }

    #[allow(dead_code)]
    pub fn get_widget_mut(&mut self, label: &str) -> Option<&mut WidgetInfo> {
        self.widgets.iter_mut().find(|w| w.label == label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_widget(label: &str) -> WidgetInfo {
        WidgetInfo {
            label: label.to_string(),
            template_id: "clock".to_string(),
            title: "Test Clock".to_string(),
            position: (100.0, 100.0),
            size: (200.0, 100.0),
            opacity: 1.0,
            always_on_top: true,
            click_through: false,
            visible: true,
        }
    }

    #[test]
    fn test_add_widget() {
        let mut state = AppState::default();
        state.add_widget(sample_widget("w1"));
        assert_eq!(state.widgets.len(), 1);
        assert_eq!(state.widgets[0].label, "w1");
    }

    #[test]
    fn test_remove_widget() {
        let mut state = AppState::default();
        state.add_widget(sample_widget("w1"));
        state.add_widget(sample_widget("w2"));
        let removed = state.remove_widget("w1");
        assert!(removed.is_some());
        assert_eq!(state.widgets.len(), 1);
        assert_eq!(state.widgets[0].label, "w2");
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut state = AppState::default();
        let removed = state.remove_widget("nonexistent");
        assert!(removed.is_none());
    }

    #[test]
    fn test_get_widget() {
        let mut state = AppState::default();
        state.add_widget(sample_widget("w1"));
        let w = state.get_widget("w1");
        assert!(w.is_some());
        assert_eq!(w.unwrap().template_id, "clock");
        assert!(state.get_widget("w99").is_none());
    }

    #[test]
    fn test_get_widget_mut() {
        let mut state = AppState::default();
        state.add_widget(sample_widget("w1"));
        if let Some(w) = state.get_widget_mut("w1") {
            w.opacity = 0.5;
        }
        assert_eq!(state.widgets[0].opacity, 0.5);
    }
}
