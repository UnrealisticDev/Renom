use std::path::{PathBuf};

pub struct Pathfinder {
    pub root: PathBuf,
}

impl Pathfinder {
    /// Get the root directory.
    pub fn root_dir(&self) -> PathBuf {
        self.root.clone()
    }

    /// Get the backup directory.
    pub fn backup_dir(&self) -> PathBuf {
        self.root.join("_renom").join("Backup")
    }

    /// Get the staging directory.
    pub fn staging_dir(&self) -> PathBuf {
        self.root.join("_renom").join("Staging")
    }

    /// Get the config directory.
    pub fn config_dir(&self) -> PathBuf {
        self.root.join("Config")
    }

    /// Get the source directory.
    pub fn source_dir(&self) -> PathBuf {
        self.root.join("Source")
    }

    /// Get the source project subdirectory.
    pub fn source_proj_dir(&self, proj_name: &str) -> PathBuf {
        self.source_dir().join(proj_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root() {
        let pathfinder = Pathfinder {
            root: PathBuf::from("TestRoot"),
        };
        assert_eq!(pathfinder.root_dir(), PathBuf::from("TestRoot\\"));
    }
    #[test]
    fn test_backup_dir() {
        let pathfinder = Pathfinder {
            root: PathBuf::from("TestRoot\\"),
        };
        assert_eq!(pathfinder.backup_dir(), PathBuf::from("TestRoot\\_renom\\Backup"));
    }

    #[test]
    fn test_staging_dir() {
        let pathfinder = Pathfinder {
            root: PathBuf::from("TestRoot\\"),
        };
        assert_eq!(
            pathfinder.staging_dir(),
            PathBuf::from("TestRoot\\_renom\\Staging")
        );
    }

    #[test]
    fn test_config_dir() {
        let pathfinder = Pathfinder {
            root: PathBuf::from("TestRoot\\"),
        };
        assert_eq!(pathfinder.config_dir(), PathBuf::from("TestRoot\\Config"));
    }

    #[test]
    fn test_source_dir() {
        let pathfinder = Pathfinder {
            root: PathBuf::from("TestRoot\\"),
        };
        assert_eq!(pathfinder.source_dir(), PathBuf::from("TestRoot\\Source"));
    }

    #[test]
    fn test_source_proj_dir() {
        let pathfinder = Pathfinder {
            root: PathBuf::from("TestRoot\\"),
        };
        assert_eq!(pathfinder.source_proj_dir("TestProject"), PathBuf::from("TestRoot\\Source\\TestProject"));
    }
}
