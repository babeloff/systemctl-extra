use std::path::PathBuf;
use clap::{ValueEnum}; 
use std::fmt;

/// Enum for selecting unit file location.
#[derive(ValueEnum, Clone, Debug)]
pub enum UnitPrivilege {
    /// Search system-wide unit file paths (e.g., /etc/systemd/system).
    System,
    /// Search user-specific unit file paths (e.g., ~/.config/systemd/user).
    User,
    /// Search a local, project-specific mock unit file path (e.g., .mycli_units).
    Local,
}

impl fmt::Display for UnitPrivilege {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnitPrivilege::System => write!(f, "System"),
            UnitPrivilege::User => write!(f, "User"),
            UnitPrivilege::Local => write!(f, "Local"),
        }
    }
}

pub trait HasPrivilege {
    /// Returns the privilege level of the unit file location.
    fn privilege(&self) -> UnitPrivilege;
}

/// Returns a list of standard systemd system unit file directories.
pub fn get_system_unit_paths() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/etc/systemd/system"),
        PathBuf::from("/usr/lib/systemd/system"),
        PathBuf::from("/lib/systemd/system"), // Often symlinks to /usr/lib/systemd/system
        // For a full systemd emulation, one might include /run/systemd/system,
        // but it's volatile and harder to mock.
    ]
}

/// Returns a list of standard systemd user unit file directories.
pub fn get_user_unit_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(config_dir) = dirs::config_dir() {
        paths.push(config_dir.join("systemd/user"));
    }
    if let Some(data_dir) = dirs::data_dir() {
        paths.push(data_dir.join("systemd/user"));
    }
    // Also common to check system-wide user units for some distros
    paths.push(PathBuf::from("/usr/lib/systemd/user"));
    paths.push(PathBuf::from("/lib/systemd/user"));
    // For a full systemd emulation, one might include /run/user/<UID>/systemd/user,
    // but it's volatile and harder to mock.
    paths
}

/// Returns the path(s) for local, project-specific unit files.
pub fn get_local_unit_paths() -> Vec<PathBuf> {
    // This is our mock directory where sample unit files are stored.
    vec![PathBuf::from("./mock_systemd_units")]
    // let mock_units_dir = PathBuf::from("./mock_systemd_units");
    // if !mock_units_dir.exists() {
    //     eprintln!("Error: Mock unit files directory not found: {}", mock_units_dir.display());
    //     eprintln!("Please create it and add some sample .service files.");
    //     return;
    // }
}

/// Helper to get all valid unit file extensions.
pub fn get_unit_file_extensions() -> &'static [&'static str] {
    &["service", "socket", "device", "mount", "automount", "swap", "target", "path", "timer", "slice", "scope"]
}

