use ini::Ini;

use crate::change::Change;

pub fn execute(changeset: Vec<Change>) {
    for change in changeset {
        match change {
            Change::RenameFile(params) => {
                std::fs::rename(params.from, params.to).expect("To handle this...")
            }
            Change::AddEntryToIni(params) => {
                let mut config = Ini::load_from_file(&params.path).unwrap();
                config
                    .with_section(Some(params.section))
                    .set(params.key, params.value);
                config.write_to_file(params.path).unwrap();
            }
        }
    }
}
