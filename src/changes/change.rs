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
        let before = params.from.clone();
        let after = params.to.clone();
        std::fs::rename(&params.from, &params.to)?;

        Ok(Box::new(move || std::fs::rename(&after, &before)))
    }

    fn replace_in_file(params: &ReplaceInFile, backup_dir: &Path) -> io::Result<Revert> {
        let before = Change::backup_file(&params.path, backup_dir)?;
        let after = params.path.clone();
        let content = std::fs::read_to_string(&params.path)?;
        let regex = Regex::new(&params.from).expect("invalid regex; coding error"); // Should panic; regex is hard-coded
        let content_after_replace = regex.replace_all(&content, params.to.as_str()).to_string();
        std::fs::write(&params.path, &content_after_replace)?;

        Ok(Box::new(move || std::fs::copy(&before, &after).map(|_| ())))
    }

    fn set_ini_entry(params: &SetIniEntry, backup_dir: &Path) -> io::Result<Revert> {
        let before = Change::backup_file(&params.path, backup_dir)?;
        let after = params.path.clone();

        let mut ini = match Ini::load_from_file(&params.path) {
            Ok(ini) => ini,
            Err(err) => match err {
                ini::ini::Error::Io(io) => return Err(io),
                ini::ini::Error::Parse(p) => return Err(io::Error::new(io::ErrorKind::Other, p)),
            },
        };
        ini.with_section(Some(&params.section))
            .set(&params.key, &params.value);
        ini.write_to_file(&params.path)?;

        Ok(Box::new(move || std::fs::copy(&before, &after).map(|_| ())))
    }

    fn append_ini_entry(params: &AppendIniEntry, backup_dir: &Path) -> io::Result<Revert> {
        let before = Change::backup_file(&params.path, backup_dir)?;
        let after = params.path.clone();

        let mut ini = match Ini::load_from_file(&params.path) {
            Ok(ini) => ini,
            Err(err) => match err {
                ini::ini::Error::Io(io) => return Err(io),
                ini::ini::Error::Parse(p) => return Err(io::Error::new(io::ErrorKind::Other, p)),
            },
        };
        ini.with_section(Some(&params.section))
            .set("dummy", "dummy");
        ini.section_mut(Some(&params.section))
            .expect("ini section missing after create") // Should panic; section created right above
            .append(&params.key, &params.value);
        ini.with_section(Some(&params.section)).delete(&"dummy");
        ini.write_to_file(&params.path)?;

        Ok(Box::new(move || std::fs::copy(&before, &after).map(|_| ())))
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
