use ini::Ini;

use crate::change::Change;

pub fn execute(changeset: Vec<Change>) {
    for change in changeset {
        match change {
            Change::RenameFile(params) => {
                println!("Renaming {:?} to {:?}", &params.from, &params.to);
                std::fs::rename(params.from, params.to).unwrap()
            }
            Change::ReplaceInFile(params) => {
                println!(
                    "Replacing {} with {} in {:?}",
                    &params.from, &params.to, &params.path
                );
                let data = std::fs::read_to_string(&params.path).unwrap();
                let data_after_replace = data.replace(&params.from, &params.to);
                std::fs::write(&params.path, data_after_replace).unwrap();
            }
            Change::SetIniEntry(params) => {
                println!(
                    "Setting ini entry [{}] {} = {} in {:?}",
                    &params.section, &params.key, &params.value, &params.path
                );
                let mut config = Ini::load_from_file(&params.path).unwrap();
                config
                    .with_section(Some(params.section))
                    .set(params.key, params.value);
                config.write_to_file(params.path).unwrap();
            }
            Change::AppendIniEntry(params) => {
                println!(
                    "Appending ini entry [{}] {} = {} in {:?}",
                    &params.section, &params.key, &params.value, &params.path
                );
                let mut config = Ini::load_from_file(&params.path).unwrap();
                config
                    .with_section(Some(&params.section))
                    .set("dummy", "dummy");
                config
                    .section_mut(Some(&params.section))
                    .unwrap()
                    .append(&params.key, &params.value);
                config.with_section(Some(&params.section)).delete(&"dummy");
                config.write_to_file(&params.path).unwrap();
            }
        }
    }
}
