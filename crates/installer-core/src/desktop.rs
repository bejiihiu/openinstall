use crate::InstallUri;

fn escape_desktop_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\n', "\\n").replace('\r', "\\r").replace(';', "\\;")
}

pub fn desktop_entry(app_name: &str, exec_path: &str, icon_name: Option<&str>) -> String {
    let app_name = escape_desktop_value(app_name);
    let exec_path = escape_desktop_value(exec_path);
    let icon_line = icon_name
        .map(|icon| format!("Icon={}\n", escape_desktop_value(icon)))
        .unwrap_or_default();
    format!(
        "[Desktop Entry]\nType=Application\nName={app_name}\nExec={exec_path} %u\n{icon_line}NoDisplay=true\nCategories=Utility;\nMimeType=x-scheme-handler/linuxinstall;x-scheme-handler/openinstall;\n"
    )
}

pub fn desktop_entry_for_install_uri(app_name: &str, exec_path: &str, uri: &InstallUri) -> String {
    let app_name = escape_desktop_value(app_name);
    let exec_path = escape_desktop_value(exec_path);
    let icon = if uri.app_id.is_empty() {
        None
    } else {
        Some(uri.app_id.as_str())
    };
    let mime_line = format!("MimeType=x-scheme-handler/{};\n", escape_desktop_value(&uri.scheme));
    let icon_line = icon
        .map(|icon| format!("Icon={}\n", escape_desktop_value(icon)))
        .unwrap_or_default();
    format!(
        "[Desktop Entry]\nType=Application\nName={app_name}\nExec={exec_path} %u\n{icon_line}NoDisplay=true\nCategories=Utility;\n{mime_line}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InstallUri;

    #[test]
    fn generates_desktop_entry() {
        let result = desktop_entry("TestApp", "/usr/bin/test", Some("test-icon"));
        assert!(result.contains("Name=TestApp"));
        assert!(result.contains("Exec=/usr/bin/test %u"));
        assert!(result.contains("Icon=test-icon"));
        assert!(result.contains("MimeType=x-scheme-handler/linuxinstall;x-scheme-handler/openinstall;"));
    }

    #[test]
    fn generates_desktop_entry_without_icon() {
        let result = desktop_entry("Minimal", "/bin/minimal", None);
        assert!(!result.contains("Icon="));
    }

    #[test]
    fn generates_desktop_entry_for_uri_scheme() {
        let uri = InstallUri::parse("openinstall://myapp").unwrap();
        let result = desktop_entry_for_install_uri("MyApp", "/usr/bin/myapp", &uri);
        assert!(result.contains("Name=MyApp"));
        assert!(result.contains("MimeType=x-scheme-handler/openinstall;"));
    }
}
