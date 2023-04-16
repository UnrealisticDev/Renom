use std::path::Path;

use crate::changes::{Change, RenameFile, SetIniEntry};

use super::Context;

/// Generate a changeset to rename an Unreal Engine project.
pub fn generate_changeset(context: &Context) -> Vec<Change> {
    let Context {
        project_root,
        project_name: old_name,
        new_name,
    } = context;

    vec![
        add_game_name_to_engine_config(project_root, new_name),
        add_project_name_to_game_config(project_root, new_name),
        rename_project_descriptor(project_root, old_name, new_name),
        rename_project_root(project_root, new_name),
    ]
}

fn rename_project_descriptor(project_root: &Path, old_name: &str, new_name: &str) -> Change {
    Change::RenameFile(RenameFile::new(
        project_root.join(old_name).with_extension("uproject"),
        project_root.join(new_name).with_extension("uproject"),
    ))
}

fn add_game_name_to_engine_config(project_root: &Path, new_name: &str) -> Change {
    Change::SetIniEntry(SetIniEntry::new(
        project_root.join("Config/DefaultEngine.ini"),
        "URL",
        "GameName",
        new_name,
    ))
}

fn add_project_name_to_game_config(project_root: &Path, new_name: &str) -> Change {
    Change::SetIniEntry(SetIniEntry::new(
        project_root.join("Config/DefaultGame.ini"),
        "/Script/EngineSettings.GeneralProjectSettings",
        "ProjectName",
        new_name,
    ))
}

fn rename_project_root(project_root: &Path, new_name: &str) -> Change {
    Change::RenameFile(RenameFile::new(
        &project_root,
        project_root.with_file_name(new_name),
    ))
}
