// Repository info
pub const REPO_OWNER: &str = "bejiihiu";
pub const REPO_NAME: &str = "openinstall";
pub const REPO_URL: &str = "https://github.com/bejiihiu/openinstall";
pub const GITHUB_API_URL: &str = "https://api.github.com/repos/bejiihiu/openinstall/releases";
pub const GITHUB_DOWNLOAD_URL: &str = "https://github.com/bejiihiu/openinstall/releases/latest/download";

// Binary/app names
pub const APP_NAME: &str = "OpenInstall";
pub const BINARY_NAME: &str = "installer";
pub const BOOTSTRAPPER_NAME: &str = "installer-bootstrapper";
pub const GUI_BINARY_NAME: &str = "installer-gui";
pub const APPLICATION_ID: &str = "io.openinstall.installer";

// Cache
pub const CACHE_DIR_NAME: &str = "openinstall";
pub const HISTORY_FILE: &str = "history.json";
pub const HISTORY_TMP_FILE: &str = "history.json.tmp";

// Network
pub const DEFAULT_SERVE_ADDR: &str = "127.0.0.1:3000";
pub const DOWNLOAD_TIMEOUT_SECS: u64 = 120;
pub const DOWNLOAD_CONNECT_TIMEOUT_SECS: u64 = 30;
pub const GITHUB_API_TIMEOUT_SECS: u64 = 30;
pub const MANIFEST_FETCH_TIMEOUT_SECS: u64 = 15;
pub const REQWEST_USER_AGENT: &str = "OpenInstall/0.1.0";

// File extensions / suffixes
pub const ARCH_SUFFIX: &str = ".pkg.tar.zst";
pub const DEB_SUFFIX: &str = ".deb";
pub const RPM_SUFFIX: &str = ".rpm";
pub const FLATPAK_SUFFIX: &str = ".flatpak";
pub const APPIMAGE_SUFFIX: &str = ".AppImage";
pub const DESKTOP_FILE_EXTENSION: &str = ".desktop";

// Paths
pub const OS_RELEASE_PATH: &str = "/etc/os-release";
pub const OS_RELEASE_FALLBACK: &str = "/usr/lib/os-release";
pub const DESKTOP_APPS_RELATIVE_PATH: &str = ".local/share/applications";
pub const LOCAL_BIN_RELATIVE_PATH: &str = ".local/bin";
pub const WINDOWS_LOCAL_BIN_ENV_VAR: &str = "LOCALAPPDATA";

// URI schemes
pub const URI_SCHEME_OPENINSTALL: &str = "openinstall";
pub const URI_SCHEME_OPENINSTALLER: &str = "openinstaller";
pub const URI_SCHEME_LINUXINSTALL: &str = "linuxinstall";
pub const URI_SCHEMES: &[&str] = &["openinstall", "openinstaller", "linuxinstall"];

// Desktop entry template
pub const DESKTOP_ENTRY_TEMPLATE: &str = "[Desktop Entry]\nType=Application\nName={name}\nComment={desc}\nExec={exec} %F\nIcon={icon}\nTerminal=false\nCategories=Utility;\nX-OpenInstall-Manifest={manifest_ref}\n";

// Desktop entry constants
pub const DESKTOP_MIME_TYPE_FORMAT: &str = "MimeType=x-scheme-handler/{};\n";
pub const DESKTOP_CATEGORIES: &str = "Utility;";
pub const DESKTOP_ENTRY_SECTION: &str = "[Desktop Entry]\n";
pub const DESKTOP_TYPE_APPLICATION: &str = "Type=Application\n";
pub const DESKTOP_NO_DISPLAY: &str = "NoDisplay=true\n";
pub const DESKTOP_ICON_FORMAT: &str = "Icon={}\n";

// Package reference object keys (for deserialization)
pub const PACKAGE_REF_KEYS: &[&str] = &["url", "href", "uri", "download", "path", "file"];

// History entry format for display
pub const HISTORY_ENTRY_FORMAT: &str = "{}. {} v{} via {} at {:?}";

// Self-update target triples
pub const TARGET_X86_64_LINUX: &str = "x86_64-unknown-linux-gnu";
pub const TARGET_AARCH64_LINUX: &str = "aarch64-unknown-linux-gnu";

// GitHub API constants
pub const GITHUB_HOST: &str = "github.com";
pub const GITHUB_API_ACCEPT_HEADER: &str = "application/vnd.github+json";
pub const GITHUB_RELEASES_API_FORMAT: &str = "https://api.github.com/repos/{owner}/{repo}/releases/latest";
pub const GITHUB_DOWNLOAD_URL_FORMAT: &str = "https://github.com/bejiihiu/openinstall/releases/latest/download/installer-{target}";

// SHA-256 constants
pub const SHA256_PREFIX: &str = "sha256:";
pub const SHA256_BUFFER_SIZE: usize = 8192;

// Signature constants
pub const ED25519_PREFIX: &str = "ed25519:";
pub const PUBLIC_KEY_HEX_LEN: usize = 64;
pub const SIGNATURE_HEX_LEN: usize = 128;
pub const KEY_FIELD_PUBLIC_KEY: &str = "public_key";
pub const KEY_FIELD_SIGNATURE: &str = "signature";

// Package manager binary names
pub const PM_APT_GET: &str = "apt-get";
pub const PM_APT: &str = "apt";
pub const PM_DPKG_QUERY: &str = "dpkg-query";
pub const PM_DNF: &str = "dnf";
pub const PM_RPM: &str = "rpm";
pub const PM_PACMAN: &str = "pacman";
pub const PM_ZYPPER: &str = "zypper";
pub const PM_FLATPAK: &str = "flatpak";
pub const PM_PKCON: &str = "pkcon";
pub const PM_APT_CACHE: &str = "apt-cache";

// Package manager command arguments
pub const PM_INSTALL_ARG: &str = "install";
pub const PM_REMOVE_ARG: &str = "remove";
pub const PM_UNINSTALL_ARG: &str = "uninstall";
pub const PM_YES_ARG: &str = "-y";
pub const PM_NOCONFIRM_ARG: &str = "--noconfirm";
pub const PM_NONINTERACTIVE_ARG: &str = "--non-interactive";
pub const PM_USER_ARG: &str = "--user";
pub const PM_LOCAL_INSTALL_ARG: &str = "install-local";
pub const PM_QUERY_ARG: &str = "-q";
pub const PM_QUERY_FORMAT_STATUS: &str = "-f=${Status}";
pub const PM_QUERY_FORMAT_VERSION: &str = "-f=${Version}";
pub const PM_QUERY_WIDE: &str = "-W";
pub const PM_QUERY_QF: &str = "--qf";
pub const PM_QUERY_FORMAT_RPM: &str = "%{VERSION}-%{RELEASE}";
pub const PM_UPGRADE_ARG: &str = "-U";
pub const PM_REMOVE_PACKAGE_ARG: &str = "-R";
pub const PM_QUERY_INFO: &str = "-Q";
pub const PM_QUERY_INFO_DETAILED: &str = "-Qi";
pub const PM_SHOW_REF_ARG: &str = "--show-ref";
pub const PM_SHOW_DEPENDENCIES_ARG: &str = "--show-dependencies";
pub const PM_DEPENDS_ARG: &str = "depends";
pub const PM_REPOQUERY_ARG: &str = "repoquery";
pub const PM_REQUIRES_ARG: &str = "--requires";
pub const PM_INFO_ARG: &str = "info";
pub const PM_GET_DETAILS_ARG: &str = "get-details";

// HTTP constants
pub const HTTP_PROTOCOLS: &[&str] = &["https://", "http://"];
pub const HTTP_METHOD_GET: &str = "GET";
pub const HTTP_HEADER_ACCEPT: &str = "Accept";
pub const HTTP_HEADER_CONTENT_TYPE: &str = "Content-Type";
pub const HTTP_HEADER_CONTENT_LENGTH: &str = "Content-Length";
pub const HTTP_HEADER_CONNECTION: &str = "Connection";
pub const HTTP_MIME_JSON: &str = "application/json";
pub const HTTP_CONNECTION_CLOSE: &str = "close";
pub const HTTP_VERSION: &str = "HTTP/1.1";

// API server constants
pub const API_LATEST_PATH: &str = "/app/latest";
pub const API_RESPONSE_OK: &str = "HTTP/1.1 200 OK\r\n";
pub const API_RESPONSE_NOT_FOUND: &str = "HTTP/1.1 404 Not Found\r\n";
pub const API_RESPONSE_BAD_REQUEST: &str = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
pub const API_JSON_NOT_FOUND: &[u8] = b"{\"error\":\"not found\"}";
pub const API_REQUEST_BUFFER_SIZE: usize = 1024;

// Script phases
pub const SCRIPT_PREINSTALL: &str = "preinstall";
pub const SCRIPT_POSTINSTALL: &str = "postinstall";
pub const SCRIPT_PREREMOVE: &str = "preremove";
pub const SCRIPT_POSTREMOVE: &str = "postremove";
pub const SCRIPT_PHASES: &[&str] = &["preinstall", "postinstall", "preremove", "postremove"];

// Shell command names
pub const SHELL_CMD_WINDOWS: &str = "cmd";
pub const SHELL_CMD_UNIX: &str = "sh";
pub const SHELL_FLAG_WINDOWS: &str = "/C";
pub const SHELL_FLAG_UNIX: &str = "-c";

// Environment variable names
pub const ENV_HOME: &str = "HOME";
pub const ENV_PATH: &str = "PATH";
pub const ENV_PATHEXT: &str = "PATHEXT";
pub const ENV_XDG_CACHE_HOME: &str = "XDG_CACHE_HOME";
pub const ENV_LOCALAPPDATA: &str = "LOCALAPPDATA";
pub const ENV_LANG: &str = "LANG";
pub const ENV_LC_ALL: &str = "LC_ALL";
pub const ENV_LC_MESSAGES: &str = "LC_MESSAGES";

// Linux standard paths
pub const USR_LOCAL_BIN: &str = "/usr/local/bin";
pub const BIN_DIR_FALLBACK: &str = "/usr/local/bin";

// URI constants
pub const URI_SCHEME_SEPARATOR: &str = "://";
pub const URI_QUERY_PARAM_M: &str = "m";
pub const URI_QUERY_PARAM_MANIFEST: &str = "manifest";

// Flatpak URI prefix
pub const FLATPAK_URI_PREFIX: &str = "flatpak://";

// Self-update minimum valid binary size
pub const SELF_UPDATE_MIN_SIZE: u64 = 4096;

// Progress/status log messages
pub const LOG_DOWNLOADING: &str = "Downloading package...";
pub const LOG_VERIFYING: &str = "Verifying package...";
pub const LOG_INSTALLING_APPIMAGE: &str = "Installing AppImage...";
pub const LOG_INSTALLING_PACKAGE: &str = "Installing package...";
pub const LOG_INSTALLING_FLATPAK: &str = "Installing flatpak: ";
pub const LOG_DONE: &str = "Done.";
pub const LOG_STDERR_PREFIX: &str = "stderr: ";

// Miscellaneous
pub const PLACEHOLDER_APP_ID: &str = "cursor";
pub const SLUG_FALLBACK: &str = "package";
pub const PACKAGE_FILE_FALLBACK_FORMAT: &str = "{}-{}.pkg";
pub const DISTRO_UNKNOWN: &str = "unknown";
pub const ARCH_UNKNOWN: &str = "unknown";
pub const MANIFEST_HINT: &str = "hint: add ?m=<manifest_url> to the URI for direct installation";
pub const LOCALE_RU_PREFIX: &str = "ru";
pub const ALERT_NO_HISTORY: &str = "No history";
pub const HISTORY_ENTRY_GUI_FORMAT: &str = "{} v{} via {} — {}";
pub const CACHE_INFO_FORMAT: &str = "{} files, {} bytes";
pub const REGISTERED_SCHEMES_MSG: &str = "URI schemes registered: openinstall://, openinstaller://, linuxinstall://";
