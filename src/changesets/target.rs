use std::path::Path;

use crate::changes::{Change, RenameFile, ReplaceInFile};

/// Generate a changeset to rename a target file. This includes the
/// following changes:
/// - Rename target class
/// - Rename target file
pub fn generate_target_changeset(
    old_name: &str,
    new_name: &str,
    project_root: impl AsRef<Path>,
) -> Vec<Change> {
    let project_root = project_root.as_ref();
    vec![
        rename_target_class(project_root, old_name, new_name),
        rename_target_file(project_root, old_name, new_name),
    ]
}

fn rename_target_class(project_root: &Path, old_name: &str, new_name: &str) -> Change {
    Change::ReplaceInFile(ReplaceInFile::new(
        project_root
            .join("Source")
            .join(old_name)
            .with_extension("Target.cs"),
        format!("{}Target", old_name),
        format!("{}Target", new_name),
    ))
}

fn rename_target_file(project_root: &Path, old_name: &str, new_name: &str) -> Change {
    Change::RenameFile(RenameFile::new(
        project_root
            .join("Source")
            .join(old_name)
            .with_extension("Target.cs"),
        project_root
            .join("Source")
            .join(new_name)
            .with_extension("Target.cs"),
    ))
}
