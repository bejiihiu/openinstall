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

pub fn t(locale: Locale, key: &str) -> &str {
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
        "manifest.signature_warning" => {
            "SIGNATURE VERIFICATION FAILED — this package may be tampered with"
        }
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
        "done.success_install" => "Application installed successfully",
        "done.success_update" => "Application updated successfully",
        "done.success_remove" => "Application removed successfully",
        "error.title" => "Error",
        "error.close" => "Close",
        "verify.sha256_ok" => "SHA256: OK",
        "verify.sha256_mismatch" => "SHA256: Mismatch",
        "verify.signature_ok" => "Signature: Valid",
        "verify.signature_invalid" => "Signature: Invalid",
        "verify.signature_missing" => "Signature: Not provided",
        "menu.verify" => "Verify",
        "menu.rollback" => "Rollback",
        "menu.reload" => "Reload",
        "menu.cache_info" => "Cache Info",
        "menu.history" => "History",
        "menu.about" => "About",
        "menu.preferences" => "Preferences",
        "settings.title" => "Preferences",
        "settings.interface" => "Interface",
        "settings.advanced" => "Advanced",
        "settings.language" => "Language",
        "settings.theme" => "Theme",
        "settings.cache_dir" => "Cache directory",
        "settings.cache_dir_subtitle" => "Where to store downloaded packages",
        "settings.download_timeout" => "Download timeout (seconds)",
        "settings.reset" => "Reset to defaults",
        "settings.save" => "Save",
        "theme.system" => "System",
        "theme.light" => "Light",
        "theme.dark" => "Dark",
        "locale.auto" => "Auto",
        "locale.en" => "English",
        "locale.ru" => "Russian",
        "status.details" => "Details",
        "status.security" => "Security",
        "status.status" => "Status",
        "toast.verified" => "Verification complete",
        "toast.cache_cleared" => "Cache info shown",
        "toast.history_shown" => "History shown",
        _ => key,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EnvGuard {
        vars: Vec<(&'static str, Option<String>)>,
    }

    impl EnvGuard {
        fn new(vars: &[&'static str]) -> Self {
            let saved = vars
                .iter()
                .map(|k| {
                    (
                        *k,
                        std::env::var_os(k).map(|v| v.to_string_lossy().to_string()),
                    )
                })
                .collect();
            EnvGuard { vars: saved }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, val) in &self.vars {
                match val {
                    Some(v) => std::env::set_var(key, v),
                    None => std::env::remove_var(key),
                }
            }
        }
    }

    #[test]
    fn locale_detect_defaults_to_en() {
        let _guard = EnvGuard::new(&["LANG", "LC_ALL", "LC_MESSAGES"]);
        std::env::remove_var("LANG");
        std::env::remove_var("LC_ALL");
        std::env::remove_var("LC_MESSAGES");
        assert_eq!(Locale::detect(), Locale::En);
    }

    #[test]
    fn locale_detect_russian() {
        let _guard = EnvGuard::new(&["LANG", "LC_ALL", "LC_MESSAGES"]);
        std::env::set_var("LANG", "ru_RU.UTF-8");
        assert_eq!(Locale::detect(), Locale::Ru);
    }

    #[test]
    fn english_translations_exist_for_all_keys() {
        let keys = [
            "app.title",
            "app.subtitle",
            "manifest.install",
            "manifest.reinstall",
            "manifest.update",
            "manifest.remove",
            "manifest.rollback",
            "manifest.verify",
            "manifest.reload",
            "manifest.cache_info",
            "manifest.history",
            "state.not_installed",
            "state.same_version",
            "state.update_available",
            "state.current_version",
            "state.available_version",
            "detail.publisher",
            "detail.version",
            "detail.license",
            "detail.homepage",
            "detail.changelog",
            "detail.architecture",
            "detail.distribution",
            "detail.package_manager",
            "detail.package",
            "detail.integrity",
            "detail.signature",
            "detail.available",
            "detail.not_set",
            "detail.not_provided",
            "detail.valid",
            "detail.invalid",
            "install.progress",
            "install.downloaded",
            "install.speed",
            "install.cancel",
            "install.logs",
            "done.title",
            "done.launch",
            "done.close",
            "done.open_folder",
            "done.success_install",
            "done.success_update",
            "done.success_remove",
            "error.title",
            "error.close",
            "verify.sha256_ok",
            "verify.sha256_mismatch",
            "verify.signature_ok",
            "verify.signature_invalid",
            "verify.signature_missing",
            "menu.verify",
            "menu.rollback",
            "menu.reload",
            "menu.cache_info",
            "menu.history",
            "menu.about",
            "status.details",
            "status.security",
            "status.status",
            "toast.verified",
            "toast.cache_cleared",
            "toast.history_shown",
        ];
        for key in keys {
            let value = t(Locale::En, key);
            assert!(!value.is_empty(), "missing EN translation for {key}");
            assert_ne!(value, key, "EN translation for {key} is the key itself");
        }
    }

    #[test]
    fn russian_translations_cover_most_keys() {
        assert_eq!(t(Locale::Ru, "app.subtitle"), "Установщик приложений Linux");
        assert_eq!(t(Locale::Ru, "manifest.install"), "Установить");
        assert_eq!(t(Locale::Ru, "done.title"), "Установка завершена");
    }

    #[test]
    fn russian_falls_back_to_english_for_missing() {
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
        let _guard = EnvGuard::new(&["LANG", "LC_ALL", "LC_MESSAGES"]);
        std::env::set_var("LANG", "en_US.UTF-8");
        std::env::set_var("LC_ALL", "ru_RU.UTF-8");
        assert_eq!(Locale::detect(), Locale::Ru);
    }
}

fn ru(key: &str) -> Option<&str> {
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
        "done.success_install" => Some("Приложение успешно установлено"),
        "done.success_update" => Some("Приложение успешно обновлено"),
        "done.success_remove" => Some("Приложение успешно удалено"),
        "error.title" => Some("Ошибка"),
        "error.close" => Some("Закрыть"),
        "verify.sha256_ok" => Some("SHA256: OK"),
        "verify.sha256_mismatch" => Some("SHA256: не совпадает"),
        "verify.signature_ok" => Some("Подпись: действительна"),
        "verify.signature_invalid" => Some("Подпись: недействительна"),
        "verify.signature_missing" => Some("Подпись: не предоставлена"),
        "menu.verify" => Some("Проверить"),
        "menu.rollback" => Some("Откатить"),
        "menu.reload" => Some("Обновить"),
        "menu.cache_info" => Some("Кэш"),
        "menu.history" => Some("История"),
        "menu.about" => Some("О приложении"),
        "menu.preferences" => Some("Настройки"),
        "settings.title" => Some("Настройки"),
        "settings.interface" => Some("Интерфейс"),
        "settings.advanced" => Some("Дополнительно"),
        "settings.language" => Some("Язык"),
        "settings.theme" => Some("Тема"),
        "settings.cache_dir" => Some("Каталог кэша"),
        "settings.cache_dir_subtitle" => Some("Где хранить скачанные пакеты"),
        "settings.download_timeout" => Some("Таймаут загрузки (секунды)"),
        "settings.reset" => Some("Сбросить настройки"),
        "settings.save" => Some("Сохранить"),
        "theme.system" => Some("Системная"),
        "theme.light" => Some("Светлая"),
        "theme.dark" => Some("Тёмная"),
        "locale.auto" => Some("Авто"),
        "locale.en" => Some("Английский"),
        "locale.ru" => Some("Русский"),
        "status.details" => Some("Подробности"),
        "status.security" => Some("Безопасность"),
        "status.status" => Some("Статус"),
        "toast.verified" => Some("Проверка завершена"),
        "toast.cache_cleared" => Some("Информация о кэше"),
        "toast.history_shown" => Some("История показана"),
        _ => None,
    }
}
