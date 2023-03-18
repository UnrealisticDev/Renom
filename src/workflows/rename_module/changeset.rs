use std::{
    fs,
    path::{Path, PathBuf},
};

use regex::Regex;

use crate::{
    changes::{AppendIniEntry, Change, RenameFile, ReplaceInFile},
    unreal::Module,
};

use super::context::Context;

/// Generate a changeset to rename an Unreal Engine module.
pub fn generate_changeset(context: &Context) -> Vec<Change> {
    let Context {
        project_root,
        project_name,
        project_targets,
        modules,
        target_module:
            Module {
                root: module_root,
                name: old_name,
                ..
            },
        target_name: new_name,
        source_with_implement_macro,
        headers_with_export_macro,
    } = context;

    let mut changeset = vec![];
    changeset.push(rename_build_class(module_root, old_name, new_name));
    changeset.push(rename_build_file(module_root, old_name, new_name));

    if let Some(source_file) = source_with_implement_macro {
        changeset.push(update_implement_macro(source_file, new_name));
    }

    changeset.extend(
        headers_with_export_macro
            .iter()
            .map(|header_file| rename_api_macro_in_header(header_file, old_name, new_name)),
    );

    changeset.push(rename_source_subfolder(module_root, new_name));

    changeset.extend(
        project_targets
            .iter()
            .map(|target_file| replace_mod_reference_in_target(target_file, old_name, new_name)),
    );

    changeset.extend(
        modules
            .iter()
            .filter(|module| &module.name != old_name)
            .map(|module| {
                replace_mod_reference_in_mod(
                    &module.root.join(&module.name).with_extension("Build.cs"),
                    old_name,
                    new_name,
                )
            }),
    );

    changeset.push(replace_mod_reference_in_project_descriptor(
        project_root,
        project_name,
        old_name,
        new_name,
    ));

    // @todo: update in plugin descriptor

    changeset.push(update_existing_redirects(project_root, old_name, new_name));
    changeset.push(append_mod_redirect(project_root, old_name, new_name));

    changeset
}

fn update_implement_macro(source_file: &PathBuf, new_name: &str) -> Change {
    let content = fs::read_to_string(&source_file).unwrap();
    let regex =
        Regex::new(r#"(?P<macro>IMPLEMENT_(GAME_|PRIMARY_GAME_)?MODULE)\((?P<impl>.+?),"#).unwrap();
    let captures = regex.captures(&content).unwrap();
    let macr = captures.name("macro").unwrap().as_str();
    let implementation = captures.name("impl").unwrap().as_str();
    Change::ReplaceInFile(ReplaceInFile::new(
        source_file,
        r#"_MODULE\(.+\)"#,
        if macr == "IMPLEMENT_PRIMARY_GAME_MODULE" {
            format!(
                r#"_MODULE({}, {}, "{}")"#,
                implementation, new_name, new_name
            )
        } else {
            format!(r#"_MODULE({}, {})"#, implementation, new_name)
        },
    ))
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

fn replace_mod_reference_in_mod(module: &Path, old_name: &str, new_name: &str) -> Change {
    Change::ReplaceInFile(ReplaceInFile::new(
        module,
        format!(r#""{}""#, old_name),
        format!(r#""{}""#, new_name),
    ))
}

fn rename_build_file(module_root: &Path, old_name: &str, new_name: &str) -> Change {
    Change::RenameFile(RenameFile::new(
        module_root.join(old_name).with_extension("Build.cs"),
        module_root.join(new_name).with_extension("Build.cs"),
    ))
}

fn rename_build_class(module_root: &Path, old_name: &str, new_name: &str) -> Change {
    Change::ReplaceInFile(ReplaceInFile::new(
        module_root.join(old_name).with_extension("Build.cs"),
        old_name,
        new_name,
    ))
}

fn rename_api_macro_in_header(header_file: &Path, old_name: &str, new_name: &str) -> Change {
    Change::ReplaceInFile(ReplaceInFile::new(
        header_file,
        format!("{}_API", old_name.to_uppercase()),
        format!("{}_API", new_name.to_uppercase()),
    ))
}

fn rename_source_subfolder(module_root: &Path, new_name: &str) -> Change {
    Change::RenameFile(RenameFile::new(
        module_root,
        module_root.with_file_name(new_name),
    ))
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
