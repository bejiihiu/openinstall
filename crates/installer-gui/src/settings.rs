use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::i18n::Locale;

const SETTINGS_FILE: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub locale: LocalePreference,
    pub theme: ThemePreference,
    pub cache_dir: String,
    pub download_timeout: u64,
    pub window_width: i32,
    pub window_height: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LocalePreference {
    Auto,
    En,
    Ru,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    System,
    Light,
    Dark,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            locale: LocalePreference::Auto,
            theme: ThemePreference::System,
            cache_dir: String::new(),
            download_timeout: 120,
            window_width: 600,
            window_height: 600,
        }
    }
}

impl AppSettings {
    pub fn load() -> Self {
        let path = settings_path();
        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = settings_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())
    }

    pub fn resolve_locale(&self) -> Locale {
        match self.locale {
            LocalePreference::Auto => Locale::detect(),
            LocalePreference::En => Locale::En,
            LocalePreference::Ru => Locale::Ru,
        }
    }
}

fn settings_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("openinstall");
    config_dir.join(SETTINGS_FILE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_are_valid() {
        let settings = AppSettings::default();
        assert_eq!(settings.locale, LocalePreference::Auto);
        assert_eq!(settings.theme, ThemePreference::System);
        assert_eq!(settings.download_timeout, 120);
        assert_eq!(settings.window_width, 600);
        assert_eq!(settings.window_height, 600);
    }

    #[test]
    fn settings_serialize_roundtrip() {
        let settings = AppSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let loaded: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings.theme, loaded.theme);
        assert_eq!(settings.locale, loaded.locale);
    }

    #[test]
    fn resolve_locale_auto_detects() {
        let settings = AppSettings {
            locale: LocalePreference::Auto,
            ..Default::default()
        };
        let _ = settings.resolve_locale();
    }

    #[test]
    fn resolve_locale_explicit_en() {
        let settings = AppSettings {
            locale: LocalePreference::En,
            ..Default::default()
        };
        assert_eq!(settings.resolve_locale(), Locale::En);
    }
}
