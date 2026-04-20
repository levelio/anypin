use crate::state::TemplateInfo;

pub fn get_builtin_templates() -> Vec<TemplateInfo> {
    vec![TemplateInfo {
        id: "clock".to_string(),
        name: "置顶时钟".to_string(),
        description: "一个简洁的置顶桌面时钟".to_string(),
        default_size: (200.0, 80.0),
    }]
}

pub fn get_template_html_path(template_id: &str) -> Option<String> {
    match template_id {
        "clock" => Some("widgets/clock/index.html".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_templates_has_clock() {
        let templates = get_builtin_templates();
        assert!(templates.iter().any(|t| t.id == "clock"));
    }

    #[test]
    fn test_clock_template_path() {
        let path = get_template_html_path("clock");
        assert_eq!(path, Some("widgets/clock/index.html".to_string()));
    }

    #[test]
    fn test_unknown_template_path() {
        let path = get_template_html_path("nonexistent");
        assert!(path.is_none());
    }
}
