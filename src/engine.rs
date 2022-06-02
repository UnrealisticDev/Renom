use ini::Ini;
use regex::Regex;

use crate::{
    changes::{AppendIniEntry, Change, RenameFile, ReplaceInFile, SetIniEntry},
    logger::Log,
};

pub fn execute(changeset: Vec<Change>) {
    for change in changeset {
        Log::process(format!("{}", change));
        let result = match change {
            Change::RenameFile(params) => rename_file(params),
            Change::ReplaceInFile(params) => replace_in_file(params),
            Change::SetIniEntry(params) => set_ini_entry(params),
            Change::AppendIniEntry(params) => append_ini_entry(params),
        };

        match result {
            Ok(_) => {}
            Err(err) => {
                Log::error(err.to_string());
                break;
            }
        }
    }
}

fn rename_file(params: RenameFile) -> std::io::Result<()> {
    std::fs::rename(params.from, params.to)?;
    Ok(())
}

fn replace_in_file(params: ReplaceInFile) -> std::io::Result<()> {
    let data = std::fs::read_to_string(&params.path)?;
    let regex = Regex::new(&params.from).unwrap(); // @todo: How do we want to handle this error?
    let data_after_replace = regex.replace_all(&data, params.to.as_str()).to_string();
    std::fs::write(&params.path, &data_after_replace)?;
    Ok(())
}

fn set_ini_entry(params: SetIniEntry) -> std::io::Result<()> {
    let mut config = Ini::load_from_file(&params.path).unwrap(); // @todo: Coerce to io result?
    config
        .with_section(Some(params.section))
        .set(params.key, params.value);
    config.write_to_file(params.path)?;
    Ok(())
}

fn append_ini_entry(params: AppendIniEntry) -> std::io::Result<()> {
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
