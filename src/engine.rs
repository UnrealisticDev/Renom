use std::path::{Path, PathBuf};

use ini::Ini;
use regex::Regex;
use sha2::{Digest, Sha256};

use crate::{
    changes::{AppendIniEntry, Change, RenameFile, ReplaceInFile, SetIniEntry},
    logger::Log,
};

type Revert = Box<dyn Fn() -> std::io::Result<()>>;

pub struct Engine {
    history: Vec<(Change, Revert)>,
}

impl Engine {
    pub fn new() -> Self {
        Self { history: vec![] }
    }
    pub fn execute(&mut self, changeset: Vec<Change>, backup_dir: impl AsRef<Path>) {
        for change in changeset {
            self.execute_single(change, backup_dir.as_ref());
        }
    }

    pub fn execute_single(&mut self, change: Change, backup_dir: &Path) {
        let result = match &change {
            Change::RenameFile(params) => rename_file(params),
            Change::ReplaceInFile(params) => replace_in_file(params, backup_dir),
            Change::SetIniEntry(params) => set_ini_entry(params, backup_dir),
            Change::AppendIniEntry(params) => append_ini_entry(params, backup_dir),
        };

        // @todo: Surface this error, so that we can revert if necessary
        match result {
            Ok(revert) => self.history.push((change, revert)),
            Err(err) => Log::error(err.to_string()),
        }
    }
}

fn rename_file(params: &RenameFile) -> std::io::Result<Revert> {
    let before = params.from.clone();
    let after = params.to.clone();

    std::fs::rename(&params.from, &params.to)?;

    Ok(Box::new(move || std::fs::rename(&after, &before)))
}

fn replace_in_file(params: &ReplaceInFile, backup_dir: &Path) -> std::io::Result<Revert> {
    let before = backup_file(&params.path, backup_dir)?;
    let after = params.path.clone();

    let data = std::fs::read_to_string(&params.path)?;
    let regex = Regex::new(&params.from).unwrap(); // @todo: How do we want to handle this error?
    let data_after_replace = regex.replace_all(&data, params.to.as_str()).to_string();
    std::fs::write(&params.path, &data_after_replace)?;

    Ok(Box::new(move || {
        std::fs::copy(&before, &after)?;
        Ok(())
    }))
}

fn set_ini_entry(params: &SetIniEntry, backup_dir: &Path) -> std::io::Result<Revert> {
    let before = backup_file(&params.path, backup_dir)?;
    let after = params.path.clone();

    let mut config = Ini::load_from_file(&params.path).unwrap(); // @todo: Coerce to io result?
    config
        .with_section(Some(&params.section))
        .set(&params.key, &params.value);
    config.write_to_file(&params.path)?;

    Ok(Box::new(move || {
        std::fs::copy(&before, &after)?;
        Ok(())
    }))
}

fn append_ini_entry(params: &AppendIniEntry, backup_dir: &Path) -> std::io::Result<Revert> {
    let before = backup_file(&params.path, backup_dir)?;
    let after = params.path.clone();

    let mut config = Ini::load_from_file(&params.path).unwrap(); // @todo: Coerce to io result?
    config
        .with_section(Some(&params.section))
        .set("dummy", "dummy");
    config
        .section_mut(Some(&params.section))
        .unwrap()
        .append(&params.key, &params.value);
    config.with_section(Some(&params.section)).delete(&"dummy");
    config.write_to_file(&params.path)?;

    Ok(Box::new(move || {
        std::fs::copy(&before, &after)?;
        Ok(())
    }))
}

fn backup_file(file: &Path, backup_dir: &Path) -> std::io::Result<PathBuf> {
    let content = std::fs::read_to_string(file)?;
    let hash = Sha256::digest(&content);
    let path = backup_dir.join(format!("{:x}", hash));
    std::fs::write(&path, &content)?;
    Ok(path)
}
