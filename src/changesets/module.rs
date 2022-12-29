use std::{
    fs,
    path::{Path, PathBuf},
};

use regex::Regex;
use walkdir::WalkDir;

use crate::changes::{AppendIniEntry, Change, RenameFile, ReplaceInFile};

/// Generate a changeset to rename a build file. This includes the
/// following changes:
/// - Rename target class
/// - Rename target file
///
/// @todo: replace module name in other modules
pub fn generate_module_changeset(
    old_name: &str,
    new_name: &str,
    project_root: impl AsRef<Path>,
    project_name: &str,
) -> Vec<Change> {
    let project_root = project_root.as_ref();
    let mut changeset = vec![
        rename_build_class(project_root, old_name, new_name),
        rename_build_file(project_root, old_name, new_name),
    ];

    // @todo: refine the replace
    if let Some(implementation_file) = find_mod_implementation(project_root, old_name) {
        update_mod_implementation(&mut changeset, implementation_file, old_name, new_name);
    }

    changeset.extend(
        get_files_including_api_macro(project_root, old_name)
            .iter()
            .map(|header| {
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

    changeset.push(replace_mod_reference_in_project_descriptor(
        project_root,
        project_name,
        old_name,
        new_name,
    ));

    changeset.push(update_existing_redirects(project_root, old_name, new_name));
    changeset.push(append_mod_redirect(project_root, old_name, new_name));

    changeset
}

fn find_mod_implementation(project_root: &Path, old_name: &str) -> Option<PathBuf> {
    WalkDir::new(project_root.join("Source").join(old_name))
        .into_iter()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path().to_owned())
        .filter(|path| path.is_file() && path.extension().map_or(false, |ext| ext == "cpp"))
        .find(|source_file| match fs::read_to_string(source_file) {
            Ok(content) => content.contains("_MODULE"),
            Err(_) => false,
        })
}

fn update_mod_implementation(
    changeset: &mut Vec<Change>,
    implementation_file: PathBuf,
    old_name: &str,
    new_name: &str,
) {
    let content = fs::read_to_string(&implementation_file).unwrap();
    let regex = Regex::new(r#"_MODULE\(.+\)"#).unwrap();
    let old_impl = regex.find(&content).map(|mat| mat.as_str()).unwrap();
    let new_impl = old_impl.replace(old_name, new_name);
    changeset.push(Change::ReplaceInFile(ReplaceInFile::new(
        implementation_file,
        old_impl.replace('(', r#"\("#).replace(')', r#"\)"#),
        new_impl,
    )))
}

fn update_existing_redirects(project_root: &Path, old_name: &str, new_name: &str) -> Change {
    Change::ReplaceInFile(ReplaceInFile::new(
        project_root.join("Config").join("DefaultEngine.ini"),
        format!(
            r#"\(OldName="(?P<old>.+?)",\s*NewName="/Script/{}"\)"#,
            old_name
        ),
        format!(r#"(OldName="$old", NewName="/Script/{}")"#, new_name),
    ))
}

fn append_mod_redirect(project_root: &Path, old_name: &str, new_name: &str) -> Change {
    Change::AppendIniEntry(AppendIniEntry::new(
        project_root.join("Config").join("DefaultEngine.ini"),
        "CoreRedirects",
        "+PackageRedirects",
        format!(
            r#"(OldName="/Script/{}",NewName="/Script/{}")"#,
            old_name, new_name
        ),
    ))
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

fn get_files_including_api_macro(project_root: &Path, mod_name: &str) -> Vec<PathBuf> {
    let files: Vec<PathBuf> = WalkDir::new(project_root.join("Source").join(mod_name))
        .into_iter()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path().to_owned())
        .filter(|path| {
            let content = fs::read_to_string(path);
            content.is_ok()
                && content
                    .unwrap()
                    .contains(&format!("{}_API", mod_name.to_uppercase()))
        })
        .filter_map(|path| {
            path.strip_prefix(project_root)
                .map(|path| path.to_owned())
                .ok()
        })
        .collect();

    files
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

fn replace_mod_reference_in_project_descriptor(
    project_root: &Path,
    project_name: &str,
    old_name: &str,
    new_name: &str,
) -> Change {
    Change::ReplaceInFile(ReplaceInFile::new(
        project_root.join(project_name).with_extension("uproject"),
        format!(r#""{}""#, old_name),
        format!(r#""{}""#, new_name),
    ))
}
