use std::path::Path;

use crate::{
    changes::{Change, RenameFile, ReplaceInFile},
    unreal::Target,
};

use super::context::Context;

/// Generate a changeset to rename an Unreal Engine target.
pub fn generate_changeset(context: &Context) -> Vec<Change> {
    let Context {
        target_target: Target {
            name: old_name,
            path: target_file,
        },
        target_name: new_name,
        ..
    } = context;

    // @todo: rename references in other targets

    vec![
        rename_target_class(target_file, old_name, new_name),
        rename_target_file(target_file, new_name),
    ]
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
