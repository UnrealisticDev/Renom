use std::path::Path;

use crate::{
    changes::{Change, RenameFile, ReplaceInFile},
    unreal::Target,
};

use super::Context;

/// Generate a changeset to rename an Unreal Engine target.
pub fn generate_changeset(context: &Context) -> Vec<Change> {
    let Context {
        project_targets,
        target: Target {
            name: old_name,
            path: target_file,
        },
        new_name,
        ..
    } = context;

    let mut changeset = vec![];

    changeset.push(rename_target_class(target_file, old_name, new_name));
    changeset.push(rename_target_file(target_file, new_name));
    changeset.extend(rename_cross_target_references(
        target_file,
        project_targets,
        old_name,
        new_name,
    ));

    changeset
}

fn rename_cross_target_references(
    target_file: &Path,
    project_targets: &[Target],
    old_name: &str,
    new_name: &str,
) -> Vec<Change> {
    project_targets
        .iter()
        .filter(|target| &&target.path != &target_file)
        .map(|target| rename_target_references_in_target(&target.path, old_name, new_name))
        .collect()
}

fn rename_target_references_in_target(
    target_file: &Path,
    old_name: &str,
    new_name: &str,
) -> Change {
    Change::ReplaceInFile(ReplaceInFile::new(
        target_file,
        format!("{}Target", old_name),
        format!("{}Target", new_name),
    ))
}

fn rename_target_class(target_file: &Path, old_name: &str, new_name: &str) -> Change {
    Change::ReplaceInFile(ReplaceInFile::new(
        target_file,
        format!("{}Target", old_name),
        format!("{}Target", new_name),
    ))
}

fn rename_target_file(target_file: &Path, new_name: &str) -> Change {
    Change::RenameFile(RenameFile::new(
        target_file,
        target_file.with_file_name(format!("{new_name}.Target.cs")),
    ))
}
