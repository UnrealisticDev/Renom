use std::path::{Path, PathBuf};

use ini::Ini;
use regex::Regex;
use sha2::{Digest, Sha256};

use crate::{
    changes::{AppendIniEntry, Change, RenameFile, ReplaceInFile, SetIniEntry},
    logger::Log,
};

// @todo: Introduce a revert mechanism
// @todo: Keep track of progress
pub fn execute(changeset: Vec<Change>, backup_dir: impl AsRef<Path>) {
    for change in &changeset {
        Log::process(format!("{}", change));
        let result = match change {
            Change::RenameFile(params) => rename_file(params),
            Change::ReplaceInFile(params) => replace_in_file(params, backup_dir.as_ref()),
            Change::SetIniEntry(params) => set_ini_entry(params, backup_dir.as_ref()),
            Change::AppendIniEntry(params) => append_ini_entry(params, backup_dir.as_ref()),
        };

        if let Err(err) = result {
            Log::error(err.to_string());
            break;
        }
    }
}

fn rename_file(params: &RenameFile) -> std::io::Result<()> {
    std::fs::rename(&params.from, &params.to)?;
    Ok(())
}

fn replace_in_file(params: &ReplaceInFile, backup_dir: &Path) -> std::io::Result<()> {
    backup_file(&params.path, backup_dir)?;
    let data = std::fs::read_to_string(&params.path)?;
    let regex = Regex::new(&params.from).unwrap(); // @todo: How do we want to handle this error?
    let data_after_replace = regex.replace_all(&data, params.to.as_str()).to_string();
    std::fs::write(&params.path, &data_after_replace)?;
    Ok(())
}

fn set_ini_entry(params: &SetIniEntry, backup_dir: &Path) -> std::io::Result<()> {
    backup_file(&params.path, backup_dir)?;
    let mut config = Ini::load_from_file(&params.path).unwrap(); // @todo: Coerce to io result?
    config
        .with_section(Some(&params.section))
        .set(&params.key, &params.value);
    config.write_to_file(&params.path)?;
    Ok(())
}

fn append_ini_entry(params: &AppendIniEntry, backup_dir: &Path) -> std::io::Result<()> {
    backup_file(&params.path, backup_dir)?;
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
    Ok(())
}

fn backup_file(file: &Path, backup_dir: &Path) -> std::io::Result<PathBuf> {
    let content = std::fs::read_to_string(file)?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let hash = hasher.finalize();
    let backup_file = backup_dir.join(format!("{:x}", hash));
    std::fs::write(&backup_file, &content)?;
    Ok(backup_file)
}
