use std::fmt;

use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PackageMatrix {
    pub arch: Option<String>,
    pub ubuntu: Option<String>,
    pub fedora: Option<String>,
    pub opensuse: Option<String>,
    pub flatpak: Option<String>,
    pub appimage: Option<String>,
    pub windows: Option<String>,
    pub macos: Option<String>,
}

impl<'de> Deserialize<'de> for PackageMatrix {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Arch,
            Ubuntu,
            Fedora,
            Opensuse,
            Flatpak,
            Appimage,
            Fallback,
            Windows,
            Macos,
            #[serde(other)]
            Ignored,
        }

        struct PackageMatrixVisitor;

        impl<'de> Visitor<'de> for PackageMatrixVisitor {
            type Value = PackageMatrix;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a map of package slots to package references")
            }

            fn visit_map<V>(self, mut map: V) -> Result<PackageMatrix, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut arch = None;
                let mut ubuntu = None;
                let mut fedora = None;
                let mut opensuse = None;
                let mut flatpak = None;
                let mut appimage = None;
                let mut fallback = None;
                let mut windows = None;
                let mut macos = None;

                while let Some(key) = map.next_key::<Field>()? {
                    let value = map.next_value::<serde_json::Value>()?;
                    let parsed = deserialize_package_ref_value(value)
                        .map_err(de::Error::custom)?;
                    match key {
                        Field::Arch => {
                            if arch.is_some() {
                                return Err(de::Error::duplicate_field("arch"));
                            }
                            arch = parsed.filter(|s| !s.is_empty());
                        }
                        Field::Ubuntu => {
                            if ubuntu.is_some() {
                                return Err(de::Error::duplicate_field("ubuntu"));
                            }
                            ubuntu = parsed.filter(|s| !s.is_empty());
                        }
                        Field::Fedora => {
                            if fedora.is_some() {
                                return Err(de::Error::duplicate_field("fedora"));
                            }
                            fedora = parsed.filter(|s| !s.is_empty());
                        }
                        Field::Opensuse => {
                            if opensuse.is_some() {
                                return Err(de::Error::duplicate_field("opensuse"));
                            }
                            opensuse = parsed.filter(|s| !s.is_empty());
                        }
                        Field::Flatpak => {
                            if flatpak.is_some() {
                                return Err(de::Error::duplicate_field("flatpak"));
                            }
                            flatpak = parsed.filter(|s| !s.is_empty());
                        }
                        Field::Appimage => {
                            if appimage.is_some() {
                                return Err(de::Error::duplicate_field("appimage"));
                            }
                            appimage = parsed.filter(|s| !s.is_empty());
                        }
                        Field::Fallback => {
                            if fallback.is_some() {
                                return Err(de::Error::duplicate_field("fallback"));
                            }
                            fallback = parsed.filter(|s| !s.is_empty());
                        }
                        Field::Windows => {
                            if windows.is_some() {
                                return Err(de::Error::duplicate_field("windows"));
                            }
                            windows = parsed.filter(|s| !s.is_empty());
                        }
                        Field::Macos => {
                            if macos.is_some() {
                                return Err(de::Error::duplicate_field("macos"));
                            }
                            macos = parsed.filter(|s| !s.is_empty());
                        }
                        Field::Ignored => {}
                    }
                }

                Ok(PackageMatrix {
                    arch,
                    ubuntu,
                    fedora,
                    opensuse,
                    flatpak,
                    appimage: appimage.or(fallback),
                    windows,
                    macos,
                })
            }
        }

        deserializer.deserialize_map(PackageMatrixVisitor)
    }
}

impl Serialize for PackageMatrix {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(None)?;
        if let Some(v) = &self.arch {
            map.serialize_entry("arch", v)?;
        }
        if let Some(v) = &self.ubuntu {
            map.serialize_entry("ubuntu", v)?;
        }
        if let Some(v) = &self.fedora {
            map.serialize_entry("fedora", v)?;
        }
        if let Some(v) = &self.opensuse {
            map.serialize_entry("opensuse", v)?;
        }
        if let Some(v) = &self.flatpak {
            map.serialize_entry("flatpak", v)?;
        }
        if let Some(v) = &self.appimage {
            map.serialize_entry("appimage", v)?;
        }
        if let Some(v) = &self.windows {
            map.serialize_entry("windows", v)?;
        }
        if let Some(v) = &self.macos {
            map.serialize_entry("macos", v)?;
        }
        map.end()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageSlot {
    Arch,
    Ubuntu,
    Fedora,
    OpenSuse,
    Flatpak,
    AppImage,
}

impl fmt::Display for PackageSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            PackageSlot::Arch => "arch",
            PackageSlot::Ubuntu => "ubuntu",
            PackageSlot::Fedora => "fedora",
            PackageSlot::OpenSuse => "opensuse",
            PackageSlot::Flatpak => "flatpak",
            PackageSlot::AppImage => "appimage",
        };

        f.write_str(name)
    }
}

fn deserialize_package_ref_value(value: serde_json::Value) -> Result<Option<String>, String> {
    match value {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::String(s) => {
            let s = s.trim().to_string();
            if s.is_empty() { Ok(None) } else { Ok(Some(s)) }
        }
        serde_json::Value::Object(map) => {
            if map.is_empty() {
                return Ok(None);
            }
            for key in ["url", "href", "uri", "download", "path", "file"] {
                if let Some(serde_json::Value::String(value)) = map.get(key) {
                    let value = value.trim().to_string();
                    if !value.is_empty() {
                        return Ok(Some(value));
                    }
                }
            }
            Err("package objects must contain a string field named url, href, uri, download, path, or file".to_string())
        }
        other => Err(format!("expected a string or object for package reference, got {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_slot_display() {
        assert_eq!(format!("{}", PackageSlot::Arch), "arch");
        assert_eq!(format!("{}", PackageSlot::Ubuntu), "ubuntu");
        assert_eq!(format!("{}", PackageSlot::Fedora), "fedora");
        assert_eq!(format!("{}", PackageSlot::OpenSuse), "opensuse");
        assert_eq!(format!("{}", PackageSlot::Flatpak), "flatpak");
        assert_eq!(format!("{}", PackageSlot::AppImage), "appimage");
    }

    #[test]
    fn package_matrix_default_is_empty() {
        let m = PackageMatrix::default();
        assert!(m.arch.is_none());
        assert!(m.ubuntu.is_none());
        assert!(m.fedora.is_none());
        assert!(m.opensuse.is_none());
        assert!(m.flatpak.is_none());
        assert!(m.appimage.is_none());
        assert!(m.windows.is_none());
        assert!(m.macos.is_none());
    }

    #[test]
    fn parses_manifest_with_object_packages() {
        let manifest = crate::Manifest::from_json_str(
            r#"
            {
                "name": "Cursor",
                "publisher": "Anysphere",
                "version": "1.5.0",
                "description": "AI Code Editor",
                "packages": {
                    "ubuntu": { "url": "https://example.com/ilovearina.deb" },
                    "arch": { "url": "https://example.com/ilovearina.pkg.tar.zst" }
                }
            }
            "#,
        )
        .expect("manifest should parse");

        assert_eq!(
            manifest.packages.ubuntu.as_deref(),
            Some("https://example.com/ilovearina.deb")
        );
        assert_eq!(
            manifest.packages.arch.as_deref(),
            Some("https://example.com/ilovearina.pkg.tar.zst")
        );
        manifest.validate().expect("manifest should validate");
    }

    #[test]
    fn fallback_is_mapped_to_appimage() {
        let manifest = crate::Manifest::from_json_str(
            r#"
            {
                "name": "Test",
                "publisher": "Pub",
                "version": "1.0",
                "description": "test",
                "packages": {
                    "fallback": "https://example.com/test.AppImage"
                }
            }
            "#,
        )
        .expect("manifest with fallback key should parse");

        assert_eq!(
            manifest.packages.appimage.as_deref(),
            Some("https://example.com/test.AppImage")
        );
    }

    #[test]
    fn parses_manifest_with_flatpak() {
        let manifest = crate::Manifest::from_json_str(
            r#"
            {
                "name": "Test",
                "publisher": "Pub",
                "version": "1.0",
                "description": "test",
                "packages": {
                    "flatpak": "flatpak://com.example.Test"
                }
            }
            "#,
        )
        .expect("manifest with flatpak should parse");

        assert_eq!(
            manifest.packages.flatpak.as_deref(),
            Some("flatpak://com.example.Test")
        );
    }
}
