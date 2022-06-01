use crate::change::{AddEntryToIni, Change, RenameFile};

/// Generate a changeset to rename a Blueprint project from the
/// old project name to the new project name. This includes the
/// following changes:
/// - Rename the project descriptor file
/// - Add a GameName entry under the URL section to the DefaultEngine.ini config file
/// - Rename the project root directory
pub fn generate_blueprint_changeset(old_project_name: &str, new_project_name: &str) -> Vec<Change> {
    let mut changeset = vec![];

    changeset.push(Change::RenameFile(RenameFile::new(
        format!("{}.uproject", old_project_name),
        format!("{}.uproject", new_project_name),
    )));

    changeset.push(Change::AddEntryToIni(AddEntryToIni::new(
        "Config/DefaultEngine.ini",
        "URL",
        "GameName",
        new_project_name,
    )));

    changeset.push(Change::RenameFile(RenameFile::new(".", new_project_name)));

    changeset
}

#[cfg(test)]
mod tests {
    use crate::change::*;

    use super::generate_blueprint_changeset;

    #[test]
    fn blueprint_changeset_is_correct() {
        let old_project_name = "Start";
        let new_project_name = "Finish";
        let changeset = generate_blueprint_changeset(old_project_name, new_project_name);
        let expected = vec![
            // Rename project descriptor
            Change::RenameFile(RenameFile::new("Start.uproject", "Finish.uproject")),
            // Add Game Name entry to ini file
            Change::AddEntryToIni(AddEntryToIni::new(
                "Config/DefaultEngine.ini",
                "URL",
                "GameName",
                "Finish",
            )),
            // Rename project root
            Change::RenameFile(RenameFile::new(".", "Finish")),
        ];

        assert_eq!(changeset, expected);
    }
}
