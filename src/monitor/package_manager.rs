use crate::external_program::program::Program;
use crate::util::data_container::DataContainer;
use anyhow::{Context, Result};
use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum PackageManagerType {
    Apt,
    Dnf,
    Pacman,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    name: String,
    version: String,
}

#[derive(Default, Debug, Clone)]
pub struct PackageManager {
    pub manager_type: Option<PackageManagerType>,
    user_packages: Option<Vec<Package>>,
}

impl PackageManager {
    pub fn new() -> Self {
        Self {
            manager_type: seek_packager_manager_type(),
            user_packages: None,
        }
    }

    pub fn detect_package(&mut self) -> Result<()> {
        let manager_type = self
            .manager_type
            .as_ref()
            .context("No package manager detected")?;

        let command = match manager_type {
            PackageManagerType::Dnf => {
                "dnf repoquery --installed --queryformat '%{name} %{version} %{reason}\n' | grep User"
            }
            PackageManagerType::Apt => {
                "apt-mark showmanual | xargs -r dpkg-query -W -f='${binary:Package} ${Version}\n'"
            }
            _ => return Ok(()),
        };

        let mut program = Program::new_command("bash").args(["-c", command]);

        program.start(Some(0))?;
        let output = program.read()?;

        // output of apt and dnf follows same format. see doc/package_manager.md for sample
        let packages: Vec<Package> = output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    Some(Package {
                        name: parts[0].to_string(),
                        version: parts[1].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        self.user_packages = Some(packages);
        Ok(())
    }

    pub fn import_package_record(&mut self, filename: Option<&str>) -> Result<()> {
        #[derive(Deserialize)]
        struct PackageRecord {
            package: Vec<Package>,
        }

        let path = match filename {
            Some(name) => {
                let mut path = PathBuf::from(name);
                if path.extension().is_none() {
                    path.set_extension("toml");
                }
                path
            }
            None => find_latest_package_record()
                .context("no package record files found in current directory")?,
        };

        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read file {}", path.display()))?;
        let record: PackageRecord = toml::from_str(&content)
            .with_context(|| format!("failed to parse TOML from {}", path.display()))?;

        self.user_packages = Some(record.package);
        Ok(())
    }

    /// if `filename` is none, the export file's name will be `installed-package-record-YYYY-MM-DD-MM-SS.toml`
    pub fn export_package_record(&self, filename: Option<&str>) -> Result<()> {
        let packages = self
            .user_packages
            .as_ref()
            .context("no user packages detected, run detect_package() first")?;

        let path = match filename {
            Some(name) => {
                let mut path = PathBuf::from(name);
                if path.extension().is_none() {
                    path.set_extension("toml");
                }
                path
            }
            None => {
                let ts = Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
                PathBuf::from(format!("installed-package-record-{ts}.toml"))
            }
        };

        let mut record: HashMap<&str, &[Package]> = HashMap::new();
        record.insert("package", packages.as_slice());
        let toml_content =
            toml::to_string(&record).context("failed to serialize package record to TOML")?;

        let mut file = fs::File::create(&path)
            .with_context(|| format!("failed to create file {}", path.display()))?;
        file.write_all(toml_content.as_bytes())?;

        Ok(())
    }

    pub fn export_package_install_list(&self, filename: &str) -> Result<()> {
        Ok(())
    }
}

fn find_latest_package_record() -> Option<PathBuf> {
    let prefix = "installed-package-record-";
    let suffix = ".toml";
    let ts_format = "%Y-%m-%d-%H-%M-%S";

    fs::read_dir(".")
        .ok()?
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(prefix) && name.ends_with(suffix) {
                let ts_str = &name[prefix.len()..name.len() - suffix.len()];
                NaiveDateTime::parse_from_str(ts_str, ts_format)
                    .ok()
                    .map(|_| entry.path())
            } else {
                None
            }
        })
        .max_by_key(|path| {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            let ts_str = &name[prefix.len()..name.len() - suffix.len()];
            NaiveDateTime::parse_from_str(ts_str, ts_format).unwrap_or_default()
        })
}

/// return package manager for the system
/// for windows and macOS, it will be None. for those linux distro that use apt as package manager, it returns apt, same to dnf and pacman.
/// see [package_managers.toml] for the relationship definition
fn seek_packager_manager_type() -> Option<PackageManagerType> {
    #[cfg(not(target_os = "linux"))]
    {
        return None;
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(os_name) = get_os_name() {
            if let Some(pm) = detect_package_manager_by_os_name(&os_name) {
                return Some(pm);
            }
        }

        detect_package_manager_by_command()
    }
}

#[cfg(target_os = "linux")]
fn package_manager_config() -> Option<PackageManagerConfig> {
    toml::from_str(include_str!("package_managers.toml")).ok()
}

#[cfg(target_os = "linux")]
fn detect_package_manager_by_os_name(os_name: &str) -> Option<PackageManagerType> {
    let os_name = os_name.to_lowercase();

    package_manager_config()?
        .os_name
        .into_iter()
        .find_map(|rule| {
            if rule.contains.iter().any(|name| os_name.contains(name)) {
                PackageManagerType::try_from(rule.package_manager.as_str()).ok()
            } else {
                None
            }
        })
}

#[cfg(target_os = "linux")]
fn detect_package_manager_by_command() -> Option<PackageManagerType> {
    for rule in package_manager_config()?.command {
        let mut program = Program::new_command(&rule.program).args(rule.args);
        if program.start(Some(0)).is_ok() && !program.read().unwrap_or_default().trim().is_empty() {
            return PackageManagerType::try_from(rule.package_manager.as_str()).ok();
        }
    }

    None
}

#[cfg(target_os = "linux")]
#[derive(Deserialize)]
struct PackageManagerConfig {
    os_name: Vec<PackageManagerOsNameRule>,
    command: Vec<PackageManagerCommandRule>,
}

#[cfg(target_os = "linux")]
#[derive(Deserialize)]
struct PackageManagerOsNameRule {
    contains: Vec<String>,
    package_manager: String,
}

#[cfg(target_os = "linux")]
#[derive(Deserialize)]
struct PackageManagerCommandRule {
    program: String,
    args: Vec<String>,
    package_manager: String,
}

impl From<&PackageManagerType> for String {
    fn from(value: &PackageManagerType) -> Self {
        match value {
            PackageManagerType::Apt => "apt".to_string(),
            PackageManagerType::Dnf => "dnf".to_string(),
            PackageManagerType::Pacman => "pacman".to_string(),
        }
    }
}

impl From<PackageManagerType> for DataContainer {
    fn from(value: PackageManagerType) -> Self {
        match value {
            PackageManagerType::Apt => DataContainer::from("apt"),
            PackageManagerType::Dnf => DataContainer::from("dnf"),
            PackageManagerType::Pacman => DataContainer::from("pacman"),
        }
    }
}

impl TryFrom<&str> for PackageManagerType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "apt" => Ok(PackageManagerType::Apt),
            "dnf" => Ok(PackageManagerType::Dnf),
            "pacman" => Ok(PackageManagerType::Pacman),
            _ => Err(()),
        }
    }
}

#[cfg(target_os = "linux")]
pub fn get_os_name() -> Option<String> {
    // https://www.linux.org/docs/man5/os-release.html
    let content = fs::read_to_string("/etc/os-release").ok()?;
    content.lines().find_map(|line| {
        let line = line.trim();
        line.strip_prefix("NAME=")
            .map(|v| v.trim().trim_matches(|c| c == '"' || c == '\'').to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn test_get_os_name() {
        let os_name = get_os_name();
        assert!(
            os_name.is_some(),
            "get_os_name() should return Some on Linux"
        );
        let name = os_name.unwrap();
        assert!(!name.is_empty(), "OS name should not be empty");
        println!("detected OS name: {name}");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_seek_packager_manager_type() {
        let result = seek_packager_manager_type();
        assert!(result.is_some(), "should detect a package manager on Linux");
        println!("detected package manager: {:?}", result.unwrap());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_detect_package() {
        let mut manager = PackageManager::new();

        // to test on fedora
        if manager.manager_type == Some(PackageManagerType::Dnf) {
            manager.detect_package().unwrap();
            let packages = manager.user_packages.as_ref().unwrap();
            assert!(
                !packages.is_empty(),
                "should detect at least one user package"
            );
            println!("detected {} user packages", packages.len());
        }
    }

    fn make_manager() -> PackageManager {
        PackageManager {
            manager_type: Some(PackageManagerType::Apt),
            user_packages: Some(vec![
                Package {
                    name: "vim".to_string(),
                    version: "2:9.1.0".to_string(),
                },
                Package {
                    name: "htop".to_string(),
                    version: "3.3.0".to_string(),
                },
            ]),
        }
    }

    #[test]
    fn test_export_package_record_with_filename() {
        let manager = make_manager();
        let path = PathBuf::from("test_export_packages.toml");

        manager
            .export_package_record(Some(path.to_str().unwrap()))
            .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert!(content.contains("[[package]]"));
        assert!(content.contains("name = \"vim\""));
        assert!(content.contains("version = \"2:9.1.0\""));
        assert!(content.contains("name = \"htop\""));
        assert!(content.contains("version = \"3.3.0\""));
    }

    #[test]
    fn test_export_package_record_default_filename() {
        let manager = make_manager();
        let ts_prefix = Local::now().format("%Y-%m-%d-%H-%M").to_string();

        manager.export_package_record(None).unwrap();

        let entries = fs::read_dir(".").unwrap();
        let mut found: Option<PathBuf> = None;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&format!("installed-package-record-{ts_prefix}"))
                && name.ends_with(".toml")
            {
                found = Some(entry.path());
                break;
            }
        }
        let path = found.expect("default-named file should be created");

        let content = fs::read_to_string(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert!(content.contains("[[package]]"));
        assert!(content.contains("name = \"vim\""));
        assert!(content.contains("name = \"htop\""));
    }

    #[test]
    fn test_export_package_record_no_packages() {
        let manager = PackageManager {
            manager_type: Some(PackageManagerType::Apt),
            user_packages: None,
        };

        let result = manager.export_package_record(Some("should_not_exist.toml"));
        assert!(result.is_err());
        assert!(!std::path::Path::new("should_not_exist.toml").exists());
    }

    #[test]
    fn test_import_package_record_with_filename() {
        let manager = make_manager();
        let path = "test_import_packages.toml";

        manager.export_package_record(Some(path)).unwrap();

        let mut imported = PackageManager {
            manager_type: Some(PackageManagerType::Apt),
            user_packages: None,
        };
        imported.import_package_record(Some(path)).unwrap();

        let _ = fs::remove_file(path);

        let packages = imported.user_packages.as_ref().unwrap();
        assert_eq!(packages.len(), 2);
        assert_eq!(packages[0].name, "vim");
        assert_eq!(packages[0].version, "2:9.1.0");
        assert_eq!(packages[1].name, "htop");
        assert_eq!(packages[1].version, "3.3.0");
    }

    #[test]
    fn test_import_package_record_default_filename() {
        let manager = make_manager();

        manager.export_package_record(None).unwrap();

        let mut imported = PackageManager {
            manager_type: Some(PackageManagerType::Apt),
            user_packages: None,
        };
        imported.import_package_record(None).unwrap();

        let ts = Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
        let _ = fs::remove_file(format!("installed-package-record-{ts}.toml"));

        let packages = imported.user_packages.as_ref().unwrap();
        assert_eq!(packages.len(), 2);
        assert_eq!(packages[0].name, "vim");
        assert_eq!(packages[1].name, "htop");
    }

    #[test]
    fn test_import_package_record_file_not_found() {
        let mut manager = PackageManager {
            manager_type: Some(PackageManagerType::Dnf),
            user_packages: None,
        };
        let result = manager.import_package_record(Some("nonexistent.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_import_package_record_no_records() {
        let mut manager = PackageManager {
            manager_type: Some(PackageManagerType::Dnf),
            user_packages: None,
        };
        let result = manager.import_package_record(None);
        assert!(result.is_err());
    }
}
