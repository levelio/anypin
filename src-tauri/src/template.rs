use crate::state::TemplateInfo;

pub fn get_builtin_templates() -> Vec<TemplateInfo> {
    vec![
        TemplateInfo {
            id: "clock".to_string(),
            name: "置顶时钟".to_string(),
            description: "一个简洁的置顶桌面时钟".to_string(),
            default_size: (200.0, 80.0),
        },
        TemplateInfo {
            id: "timer".to_string(),
            name: "番茄钟".to_string(),
            description: "番茄钟与倒计时工具".to_string(),
            default_size: (240.0, 140.0),
        },
        TemplateInfo {
            id: "notes".to_string(),
            name: "便签".to_string(),
            description: "桌面便签，自动保存".to_string(),
            default_size: (240.0, 180.0),
        },
        TemplateInfo {
            id: "status-monitor".to_string(),
            name: "状态监控".to_string(),
            description: "服务状态指示面板".to_string(),
            default_size: (220.0, 140.0),
        },
    ]
}

pub fn get_template_html_path(template_id: &str) -> Option<String> {
    match template_id {
        "clock" => Some("widgets/clock/index.html".to_string()),
        "timer" => Some("widgets/timer/index.html".to_string()),
        "notes" => Some("widgets/notes/index.html".to_string()),
        "status-monitor" => Some("widgets/status-monitor/index.html".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_templates_count() {
        let templates = get_builtin_templates();
        assert_eq!(templates.len(), 4);
        assert!(templates
            .iter()
            .all(|t| get_template_html_path(&t.id).is_some()));
    }

    #[test]
    fn test_all_template_paths() {
        assert_eq!(
            get_template_html_path("clock"),
            Some("widgets/clock/index.html".to_string())
        );
        assert_eq!(
            get_template_html_path("timer"),
            Some("widgets/timer/index.html".to_string())
        );
        assert_eq!(
            get_template_html_path("notes"),
            Some("widgets/notes/index.html".to_string())
        );
        assert_eq!(
            get_template_html_path("status-monitor"),
            Some("widgets/status-monitor/index.html".to_string())
        );
    }

    #[test]
    fn test_unknown_template_path() {
        assert!(get_template_html_path("nonexistent").is_none());
    }
}
