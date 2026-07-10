use serde::Deserialize;
use crate::external_program::program::Program;
use crate::util::data_container::DataContainer;

#[derive(Debug, Clone)]
pub enum PackageManagerType {
    Apt,
    Dnf,
    Pacman,
}

#[derive(Default, Debug, Clone)]
pub struct Package {
    name: String,
    version: String,
}

#[derive(Default, Debug, Clone)]
pub struct PackageManager {
    pub manager_type: Option<PackageManagerType>,
    user_packages: Vec<Package>,
}

impl PackageManager {
    pub fn new() -> Self {
        Self {
            manager_type: seek_packager_manager_type(),
            user_packages: Vec::new(),
        }
    }
}

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

    package_manager_config()?.os_name.into_iter().find_map(|rule| {
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

impl From<PackageManagerType> for String {
    fn from(value: PackageManagerType) -> Self {
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
    let content = std::fs::read_to_string("/etc/os-release").ok()?;
    content
        .lines()
        .find_map(|line| {
            let line = line.trim();
            line.strip_prefix("NAME=")
                .map(|v| v.trim().trim_matches(|c| c == '"' || c == '\'').to_string())
        })
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    #[test]
    fn test_get_os_name() {
        let os_name = super::get_os_name();
        assert!(os_name.is_some(), "get_os_name() should return Some on Linux");
        let name = os_name.unwrap();
        assert!(!name.is_empty(), "OS name should not be empty");
        println!("detected OS name: {name}");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_seek_packager_manager_type() {
        let result = super::seek_packager_manager_type();
        assert!(result.is_some(), "should detect a package manager on Linux");
        println!("detected package manager: {:?}", result.unwrap());
    }

    #[test]
    fn test_package_manager_type_into_string() {
        let apt: String = super::PackageManagerType::Apt.into();
        let dnf: String = super::PackageManagerType::Dnf.into();
        let pacman: String = super::PackageManagerType::Pacman.into();

        assert_eq!(apt, "apt");
        assert_eq!(dnf, "dnf");
        assert_eq!(pacman, "pacman");
    }
}
