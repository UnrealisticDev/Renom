use std::{
    io,
    path::{Path, PathBuf},
};

use ini::Ini;
use regex::Regex;
use sha2::{Digest, Sha256};

use crate::{
    changes::{AppendIniEntry, Change, RenameFile, ReplaceInFile, SetIniEntry},
    presentation::log,
};

type Revert = Box<dyn Fn() -> io::Result<()>>;

pub struct Engine {
    history: Vec<(Change, Revert)>,
}

impl Engine {
    pub fn new() -> Self {
        Self { history: vec![] }
    }

    /// Execute a series of changes in sequential order and stores the
    /// applied changes in history with appropriate revert actions.
    /// Upon error, it will halt execution and return the error.
    pub fn execute(
        &mut self,
        changeset: Vec<Change>,
        backup_dir: impl AsRef<Path>,
    ) -> Result<(), String> {
        for change in changeset {
            log::basic(format!("Apply: {}", change));
            self.execute_single(change, backup_dir.as_ref())?;
        }
        Ok(())
    }

    fn execute_single(&mut self, change: Change, backup_dir: &Path) -> Result<(), String> {
        let result = match &change {
            Change::RenameFile(params) => rename_file(params),
            Change::ReplaceInFile(params) => replace_in_file(params, backup_dir),
            Change::SetIniEntry(params) => set_ini_entry(params, backup_dir),
            Change::AppendIniEntry(params) => append_ini_entry(params, backup_dir),
        };

        match result {
            Ok(revert) => {
                self.history.push((change, revert));
                Ok(())
            }
            Err(err) => Err(err.to_string()),
        }
    }

    /// Revert entire history of actions.
    /// Upon error, it will halt execution and return the error.
    pub fn revert(&mut self) -> Result<(), String> {
        while let Some((change, revert)) = self.history.pop() {
            log::basic(format!("Revert: {}", change));
            revert().map_err(|err| err.to_string())?;
        }
        Ok(())
    }
}

fn rename_file(params: &RenameFile) -> io::Result<Revert> {
    let before = params.from.clone();
    let after = params.to.clone();

    std::fs::rename(&params.from, &params.to)?;

    Ok(Box::new(move || std::fs::rename(&after, &before)))
}

fn replace_in_file(params: &ReplaceInFile, backup_dir: &Path) -> io::Result<Revert> {
    let before = backup_file(&params.path, backup_dir)?;
    let after = params.path.clone();

    let content = std::fs::read_to_string(&params.path)?;
    let regex = Regex::new(&params.from).expect("invalid regex; coding error"); // Should panic; regex is hard-coded
    let content_after_replace = regex.replace_all(&content, params.to.as_str()).to_string();
    std::fs::write(&params.path, &content_after_replace)?;

    Ok(Box::new(move || {
        std::fs::copy(&before, &after)?;
        Ok(())
    }))
}

fn set_ini_entry(params: &SetIniEntry, backup_dir: &Path) -> io::Result<Revert> {
    let before = backup_file(&params.path, backup_dir)?;
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

    Ok(Box::new(move || {
        std::fs::copy(&before, &after)?;
        Ok(())
    }))
}

fn append_ini_entry(params: &AppendIniEntry, backup_dir: &Path) -> io::Result<Revert> {
    let before = backup_file(&params.path, backup_dir)?;
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

    Ok(Box::new(move || {
        std::fs::copy(&before, &after)?;
        Ok(())
    }))
}

fn backup_file(file: &Path, backup_dir: &Path) -> io::Result<PathBuf> {
    let content = std::fs::read_to_string(file)?;
    let hash = Sha256::digest(&content);
    let path = backup_dir.join(format!("{:x}", hash));
    std::fs::write(&path, &content)?;
    Ok(path)
}
