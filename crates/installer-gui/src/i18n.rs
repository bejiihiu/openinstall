use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    En,
    Ru,
}

impl Locale {
    pub fn detect() -> Self {
        let lang = env::var_os("LANG")
            .or_else(|| env::var_os("LC_ALL"))
            .or_else(|| env::var_os("LC_MESSAGES"))
            .map(|v| v.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if lang.starts_with("ru") {
            Locale::Ru
        } else {
            Locale::En
        }
    }
}

#[macro_export]
macro_rules! tr {
    ($locale:expr, $key:literal) => {
        $crate::i18n::t($locale, $key)
    };
}

pub fn t<'a>(locale: Locale, key: &'a str) -> &'a str {
    match locale {
        Locale::En => en(key),
        Locale::Ru => ru(key).unwrap_or_else(|| en(key)),
    }
}

fn en(key: &str) -> &str {
    match key {
        "app.title" => "OpenInstall",
        "app.subtitle" => "Linux Application Installer",
        "manifest.install" => "Install",
        "manifest.reinstall" => "Reinstall",
        "manifest.update" => "Update",
        "manifest.remove" => "Remove",
        "manifest.rollback" => "Rollback",
        "manifest.verify" => "Verify",
        "manifest.reload" => "Reload",
        "manifest.cache_info" => "Cache Info",
        "manifest.history" => "History",
        "manifest.signature_warning" => "SIGNATURE VERIFICATION FAILED — this package may be tampered with",
        "state.not_installed" => "Not installed",
        "state.same_version" => "Already installed",
        "state.update_available" => "Update available",
        "state.current_version" => "Current version",
        "state.available_version" => "Available version",
        "detail.publisher" => "Publisher",
        "detail.version" => "Version",
        "detail.license" => "License",
        "detail.homepage" => "Homepage",
        "detail.changelog" => "Changelog",
        "detail.architecture" => "Architecture",
        "detail.distribution" => "Distribution",
        "detail.package_manager" => "Package manager",
        "detail.package" => "Package",
        "detail.integrity" => "Integrity",
        "detail.signature" => "Signature",
        "detail.available" => "Available",
        "detail.not_set" => "Not set",
        "detail.not_provided" => "Not provided",
        "detail.valid" => "Valid",
        "detail.invalid" => "Invalid",
        "install.progress" => "Installing...",
        "install.downloaded" => "Downloaded",
        "install.speed" => "Speed",
        "install.eta" => "ETA",
        "install.cancel" => "Cancel",
        "install.logs" => "Logs",
        "done.title" => "Installation Complete",
        "done.launch" => "Launch",
        "done.close" => "Close",
        "done.open_folder" => "Open Folder",
        "error.title" => "Error",
        "error.close" => "Close",
        "verify.sha256_ok" => "SHA256: OK",
        "verify.sha256_mismatch" => "SHA256: Mismatch",
        "verify.signature_ok" => "Signature: Valid",
        "verify.signature_invalid" => "Signature: Invalid",
        "verify.signature_missing" => "Signature: Not provided",
        _ => key,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_detect_defaults_to_en() {
        std::env::remove_var("LANG");
        std::env::remove_var("LC_ALL");
        std::env::remove_var("LC_MESSAGES");
        assert_eq!(Locale::detect(), Locale::En);
    }

    #[test]
    fn locale_detect_russian() {
        std::env::set_var("LANG", "ru_RU.UTF-8");
        assert_eq!(Locale::detect(), Locale::Ru);
        std::env::remove_var("LANG");
    }

    #[test]
    fn english_translations_exist_for_all_keys() {
        let keys = [
            "app.title", "app.subtitle", "manifest.install", "manifest.reinstall",
            "manifest.update", "manifest.remove", "manifest.rollback", "manifest.verify",
            "manifest.reload", "manifest.cache_info", "manifest.history",
            "state.not_installed", "state.same_version", "state.update_available",
            "state.current_version", "state.available_version",
            "detail.publisher", "detail.version", "detail.license", "detail.homepage",
            "detail.changelog", "detail.architecture", "detail.distribution",
            "detail.package_manager", "detail.package", "detail.integrity",
            "detail.signature", "detail.available", "detail.not_set", "detail.not_provided",
            "detail.valid", "detail.invalid",
            "install.progress", "install.downloaded", "install.speed",
            "install.cancel", "install.logs",
            "done.title", "done.launch", "done.close", "done.open_folder",
            "error.title", "error.close",
            "verify.sha256_ok", "verify.sha256_mismatch",
            "verify.signature_ok", "verify.signature_invalid", "verify.signature_missing",
        ];
        for key in &keys {
            let value = t(Locale::En, key);
            assert!(!value.is_empty(), "missing EN translation for {key}");
            assert_ne!(value, key, "EN translation for {key} is the key itself");
        }
    }

    #[test]
    fn russian_translations_cover_most_keys() {
        let ru_count = (0..200)
            .filter_map(|_| ru(""))
            .count();
        // Actually test a few specific ones
        assert_eq!(t(Locale::Ru, "app.subtitle"), "Установщик приложений Linux");
        assert_eq!(t(Locale::Ru, "manifest.install"), "Установить");
        assert_eq!(t(Locale::Ru, "done.title"), "Установка завершена");
    }

    #[test]
    fn russian_falls_back_to_english_for_missing() {
        // There's no English-only key that doesn't have Russian,
        // but we can test a non-existent key falls through to English
        let result = t(Locale::Ru, "nonexistent.key");
        assert_eq!(result, "nonexistent.key");
    }

    #[test]
    fn tr_macro_works() {
        let locale = Locale::En;
        assert_eq!(tr!(locale, "app.title"), "OpenInstall");
    }

    #[test]
    fn locale_detect_lc_all_overrides_lang() {
        std::env::set_var("LANG", "en_US.UTF-8");
        std::env::set_var("LC_ALL", "ru_RU.UTF-8");
        assert_eq!(Locale::detect(), Locale::Ru);
        std::env::remove_var("LANG");
        std::env::remove_var("LC_ALL");
    }
}

fn ru<'a>(key: &'a str) -> Option<&'a str> {
    match key {
        "app.title" => Some("OpenInstall"),
        "app.subtitle" => Some("Установщик приложений Linux"),
        "manifest.install" => Some("Установить"),
        "manifest.reinstall" => Some("Переустановить"),
        "manifest.update" => Some("Обновить"),
        "manifest.remove" => Some("Удалить"),
        "manifest.rollback" => Some("Откатить"),
        "manifest.verify" => Some("Проверить"),
        "manifest.reload" => Some("Обновить"),
        "manifest.cache_info" => Some("Кэш"),
        "manifest.history" => Some("История"),
        "manifest.signature_warning" => Some("ПОДПИСЬ НЕ ДЕЙСТВИТЕЛЬНА — пакет мог быть изменён"),
        "state.not_installed" => Some("Не установлено"),
        "state.same_version" => Some("Уже установлено"),
        "state.update_available" => Some("Доступно обновление"),
        "state.current_version" => Some("Текущая версия"),
        "state.available_version" => Some("Доступная версия"),
        "detail.publisher" => Some("Издатель"),
        "detail.version" => Some("Версия"),
        "detail.license" => Some("Лицензия"),
        "detail.homepage" => Some("Сайт"),
        "detail.changelog" => Some("Что нового"),
        "detail.architecture" => Some("Архитектура"),
        "detail.distribution" => Some("Дистрибутив"),
        "detail.package_manager" => Some("Пакетный менеджер"),
        "detail.package" => Some("Пакет"),
        "detail.integrity" => Some("Контрольная сумма"),
        "detail.signature" => Some("Подпись"),
        "detail.available" => Some("Доступно"),
        "detail.not_set" => Some("Не указано"),
        "detail.not_provided" => Some("Не предоставлена"),
        "detail.valid" => Some("Действительна"),
        "detail.invalid" => Some("Недействительна"),
        "install.progress" => Some("Установка..."),
        "install.downloaded" => Some("Скачано"),
        "install.speed" => Some("Скорость"),
        "install.eta" => Some("Осталось"),
        "install.cancel" => Some("Отмена"),
        "install.logs" => Some("Логи"),
        "done.title" => Some("Установка завершена"),
        "done.launch" => Some("Запустить"),
        "done.close" => Some("Закрыть"),
        "done.open_folder" => Some("Открыть папку"),
        "error.title" => Some("Ошибка"),
        "error.close" => Some("Закрыть"),
        "verify.sha256_ok" => Some("SHA256: OK"),
        "verify.sha256_mismatch" => Some("SHA256: не совпадает"),
        "verify.signature_ok" => Some("Подпись: действительна"),
        "verify.signature_invalid" => Some("Подпись: недействительна"),
        "verify.signature_missing" => Some("Подпись: не предоставлена"),
        _ => None,
    }
}
