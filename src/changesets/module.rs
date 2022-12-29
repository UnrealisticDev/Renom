use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::changes::{Change, RenameFile, ReplaceInFile};

/// Generate a changeset to rename a build file. This includes the
/// following changes:
/// - Rename target class
/// - Rename target file
/// @todo: do a better job of finding module header (or skip)
/// @todo: do a better job of finding module source (or skip)
/// @todo: do a better job of finding module definition
pub fn generate_module_changeset(
    old_name: &str,
    new_name: &str,
    project_root: impl AsRef<Path>,
    api_reference_files: &[PathBuf],
) -> Vec<Change> {
    let project_root = project_root.as_ref();
    let mut changeset = vec![
        rename_build_class(project_root, old_name, new_name),
        rename_build_file(project_root, old_name, new_name),
        rename_definition(project_root, old_name, new_name),
        rename_header_file(project_root, old_name, new_name),
        rename_source_file(project_root, old_name, new_name),
    ];

    changeset.extend(
        api_reference_files.iter().map(|header| {
            replace_api_macro_in_header_file(project_root, header, old_name, new_name)
        }),
    );

    changeset.push(rename_source_subfolder(project_root, old_name, new_name));

    find_target_file_names(project_root)
        .iter()
        .for_each(|target_name| {
            let target = project_root
                .join("Source")
                .join(target_name)
                .with_extension("Target.cs");
            changeset.push(replace_mod_reference_in_target(&target, old_name, new_name))
        });
    // @todo: mod references in project descriptor
    // @todo: add redirects

    changeset
}

fn replace_mod_reference_in_target(target: &Path, old_name: &str, new_name: &str) -> Change {
    Change::ReplaceInFile(ReplaceInFile::new(
        target,
        format!(r#""{}""#, old_name),
        format!(r#""{}""#, new_name),
    ))
}

fn rename_build_file(
    project_root: &Path,
    old_project_name: &str,
    new_project_name: &str,
) -> Change {
    Change::RenameFile(RenameFile::new(
        project_root
            .join("Source")
            .join(old_project_name)
            .join(old_project_name)
            .with_extension("Build.cs"),
        project_root
            .join("Source")
            .join(old_project_name)
            .join(new_project_name)
            .with_extension("Build.cs"),
    ))
}

fn rename_build_class(
    project_root: &Path,
    old_project_name: &str,
    new_project_name: &str,
) -> Change {
    Change::ReplaceInFile(ReplaceInFile::new(
        project_root
            .join("Source")
            .join(old_project_name)
            .join(old_project_name)
            .with_extension("Build.cs"),
        old_project_name,
        new_project_name,
    ))
}

fn rename_header_file(
    project_root: &Path,
    old_project_name: &str,
    new_project_name: &str,
) -> Change {
    Change::RenameFile(RenameFile::new(
        project_root
            .join("Source")
            .join(old_project_name)
            .join(old_project_name)
            .with_extension("h"),
        project_root
            .join("Source")
            .join(old_project_name)
            .join(new_project_name)
            .with_extension("h"),
    ))
}

fn rename_definition(
    project_root: &Path,
    old_project_name: &str,
    new_project_name: &str,
) -> Change {
    Change::ReplaceInFile(ReplaceInFile::new(
        project_root
            .join("Source")
            .join(old_project_name)
            .join(old_project_name)
            .with_extension("cpp"),
        old_project_name,
        new_project_name,
    ))
}

fn rename_source_file(
    project_root: &Path,
    old_project_name: &str,
    new_project_name: &str,
) -> Change {
    Change::RenameFile(RenameFile::new(
        project_root
            .join("Source")
            .join(old_project_name)
            .join(old_project_name)
            .with_extension("cpp"),
        project_root
            .join("Source")
            .join(old_project_name)
            .join(new_project_name)
            .with_extension("cpp"),
    ))
}

fn replace_api_macro_in_header_file(
    project_root: &Path,
    header: &Path,
    old_project_name: &str,
    new_project_name: &str,
) -> Change {
    Change::ReplaceInFile(ReplaceInFile::new(
        project_root.join(header),
        format!("{}_API", old_project_name.to_uppercase()),
        format!("{}_API", new_project_name.to_uppercase()),
    ))
}

fn rename_source_subfolder(
    project_root: &Path,
    old_project_name: &str,
    new_project_name: &str,
) -> Change {
    Change::RenameFile(RenameFile::new(
        project_root.join("Source").join(old_project_name),
        project_root.join("Source").join(new_project_name),
    ))
}

fn find_target_file_names(project_root: &Path) -> Vec<String> {
    fs::read_dir(project_root.join("Source"))
        .expect("could not read source dir")
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            entry
                .file_name()
                .to_str()
                .and_then(|filename| filename.strip_suffix(".Target.cs"))
                .map(|filename| filename.to_string())
        })
        .collect()
}
