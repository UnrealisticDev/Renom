use std::{
    fmt::Display,
    io,
    path::{Path, PathBuf},
};

use ini::Ini;
use regex::Regex;
use sha2::{Digest, Sha256};

use super::{rename_file::RenameFile, AppendIniEntry, ReplaceInFile, SetIniEntry};

#[derive(Debug, PartialEq)]
pub enum Change {
    RenameFile(RenameFile),
    ReplaceInFile(ReplaceInFile),
    SetIniEntry(SetIniEntry),
    AppendIniEntry(AppendIniEntry),
}

impl Change {
    pub fn apply(&self, backup_dir: &Path) -> io::Result<Revert> {
        match &*self {
            Change::RenameFile(params) => Change::rename_file(&params),
            Change::ReplaceInFile(params) => Change::replace_in_file(params, backup_dir),
            Change::SetIniEntry(params) => Change::set_ini_entry(params, backup_dir),
            Change::AppendIniEntry(params) => Change::append_ini_entry(params, backup_dir),
        }
    }

    fn rename_file(params: &RenameFile) -> io::Result<Revert> {
        let from = params.from.clone();
        let to = params.to.clone();
        std::fs::rename(&from, &to)?;

        Ok(Box::new(move || std::fs::rename(&to, &from)))
    }

    fn replace_in_file(params: &ReplaceInFile, backup_dir: &Path) -> io::Result<Revert> {
        let backup = Change::backup_file(&params.path, backup_dir)?;
        let target = params.path.clone();
        let content = std::fs::read_to_string(&target)?;
        let regex = Regex::new(&params.from).expect("regex should be valid");
        let content_after_replace = regex.replace_all(&content, params.to.as_str()).to_string();
        std::fs::write(&target, &content_after_replace)?;

        Ok(Box::new(move || {
            std::fs::copy(&backup, &target).map(|_| ())
        }))
    }

    fn set_ini_entry(params: &SetIniEntry, backup_dir: &Path) -> io::Result<Revert> {
        let SetIniEntry {
            section,
            key,
            value,
            path,
        } = params;

        let backup = Change::backup_file(path, backup_dir)?;
        let target = path.clone();

        let mut ini = match Ini::load_from_file(&target) {
            Ok(ini) => ini,
            Err(err) => match err {
                ini::ini::Error::Io(io) => return Err(io),
                ini::ini::Error::Parse(p) => return Err(io::Error::new(io::ErrorKind::Other, p)),
            },
        };
        ini.with_section(Some(section)).set(key, value);
        ini.write_to_file(&target)?;

        Ok(Box::new(move || {
            std::fs::copy(&backup, &target).map(|_| ())
        }))
    }

    fn append_ini_entry(params: &AppendIniEntry, backup_dir: &Path) -> io::Result<Revert> {
        let AppendIniEntry {
            section,
            key,
            value,
            path,
        } = params;

        let backup = Change::backup_file(path, backup_dir)?;
        let target = path.clone();

        let mut ini = match Ini::load_from_file(&target) {
            Ok(ini) => ini,
            Err(err) => match err {
                ini::ini::Error::Io(io) => return Err(io),
                ini::ini::Error::Parse(p) => return Err(io::Error::new(io::ErrorKind::Other, p)),
            },
        };
        ini.with_section(Some(section)).set("dummy", "dummy"); // create if does not exist
        ini.section_mut(Some(section)).unwrap().append(key, value);
        ini.with_section(Some(section)).delete(&"dummy");
        ini.write_to_file(&params.path)?;

        Ok(Box::new(move || {
            std::fs::copy(&backup, &target).map(|_| ())
        }))
    }

    fn backup_file(file: &Path, backup_dir: &Path) -> io::Result<PathBuf> {
        let content = std::fs::read_to_string(file)?;
        let hash = Sha256::digest(&content);
        let path = backup_dir.join(format!("{:x}", hash));
        std::fs::write(&path, &content)?;
        Ok(path)
    }
}

impl Display for Change {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &*self {
            Change::RenameFile(params) => write!(f, "{}", &params),
            Change::ReplaceInFile(params) => write!(f, "{}", &params),
            Change::SetIniEntry(params) => write!(f, "{}", &params),
            Change::AppendIniEntry(params) => write!(f, "{}", &params),
        }
    }
}

pub type Revert = Box<dyn Fn() -> io::Result<()>>;
