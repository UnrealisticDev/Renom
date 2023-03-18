use std::path::Path;

use crate::{
    changes::{Change, RenameFile, SetIniEntry},
    workflows::rename_project::context::Context,
};

/// Generate a changeset to rename a Blueprint project from the
/// old project name to the new project name. This includes the
/// following changes:
/// - Rename the project descriptor file
/// - Add a GameName entry under the URL section to the DefaultEngine.ini config file
/// - Add a ProjectName entry under the GeneralProjectSettings section to the DefaultGame.ini config file
/// - Rename the project root directory
pub fn generate_blueprint_changeset(context: &Context) -> Vec<Change> {
    let Context {
        project_root,
        project_name: old_project_name,
        target_name: new_project_name,
        ..
    } = context;

    vec![
        rename_project_descriptor(project_root, old_project_name, new_project_name),
        add_game_name_to_engine_config(project_root, new_project_name),
        add_project_name_to_game_config(project_root, new_project_name),
        rename_project_root(project_root, new_project_name),
    ]
}

fn rename_project_descriptor(
    project_root: &Path,
    old_project_name: &str,
    new_project_name: &str,
) -> Change {
    Change::RenameFile(RenameFile::new(
        project_root
            .join(old_project_name)
            .with_extension("uproject"),
        project_root
            .join(new_project_name)
            .with_extension("uproject"),
    ))
}

fn add_game_name_to_engine_config(project_root: &Path, new_project_name: &str) -> Change {
    Change::SetIniEntry(SetIniEntry::new(
        project_root.join("Config/DefaultEngine.ini"),
        "URL",
        "GameName",
        new_project_name,
    ))
}

fn add_project_name_to_game_config(project_root: &Path, new_project_name: &str) -> Change {
    Change::SetIniEntry(SetIniEntry::new(
        project_root.join("Config/DefaultGame.ini"),
        "/Script/EngineSettings.GeneralProjectSettings",
        "ProjectName",
        new_project_name,
    ))
}

fn rename_project_root(project_root: &Path, new_project_name: &str) -> Change {
    Change::RenameFile(RenameFile::new(
        &project_root,
        project_root.with_file_name(new_project_name),
    ))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        changes::*,
        workflows::rename_project::context::{Context, ProjectType},
    };

    use super::generate_blueprint_changeset;

    #[test]
    fn blueprint_changeset_is_correct() {
        let changeset = generate_blueprint_changeset(&Context {
            project_root: PathBuf::from(""),
            project_name: "Start".into(),
            project_type: ProjectType::Blueprint,
            target_name: "Finish".into(),
        });
        let expected = vec![
            // Rename project descriptor
            Change::RenameFile(RenameFile::new("Start.uproject", "Finish.uproject")),
            // Add Game Name entry to ini file
            Change::SetIniEntry(SetIniEntry::new(
                "Config/DefaultEngine.ini",
                "URL",
                "GameName",
                "Finish",
            )),
            // Add Project Name entry to ini file
            Change::SetIniEntry(SetIniEntry::new(
                "Config/DefaultGame.ini",
                "/Script/EngineSettings.GeneralProjectSettings",
                "ProjectName",
                "Finish",
            )),
            // Rename project root
            Change::RenameFile(RenameFile::new("", "Finish")),
        ];

        assert_eq!(changeset, expected);
    }
}