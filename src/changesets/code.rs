use std::path::{Path, PathBuf};

use crate::changes::{AppendIniEntry, Change, RenameFile, ReplaceInFile, SetIniEntry};

/// Generate a changeset to rename a code project from the
/// old project name to the new project name. This includes the
/// following changes:
/// - Replace old name with new name in project descriptor file
/// - Rename the project descriptor file
/// - Replace old name with new name in executable target file
/// - Rename executable target file
/// - Replace old name with new name in editor target file
/// - Rename editor target file
/// - Replace old name with new name in game module build file
/// - Rename game module build file
/// - Replace old API references in header files
/// - Rename game module header file
/// - Rename game module source file
/// - Rename source subfolder
/// - Update existing redirect entries in DefaultEngine config file
/// - Append redirect entry to DefaultEngine config file
/// - Add a GameName entry under the URL section to the DefaultEngine.ini config file
/// - Rename project root directory
pub fn generate_code_changeset(
    old_project_name: &str,
    new_project_name: &str,
    project_root: impl AsRef<Path>,
    api_reference_files: Vec<PathBuf>,
) -> Vec<Change> {
    let project_root = project_root.as_ref();
    let mut changeset = vec![];

    changeset.extend(vec![
        replace_in_project_descriptor(project_root, old_project_name, new_project_name),
        rename_project_descriptor(project_root, old_project_name, new_project_name),
        replace_in_exec_target_file(project_root, old_project_name, new_project_name),
        rename_exec_target_file(project_root, old_project_name, new_project_name),
        replace_in_ed_target_file(project_root, old_project_name, new_project_name),
        rename_ed_target_file(project_root, old_project_name, new_project_name),
        replace_in_mod_build_file(project_root, old_project_name, new_project_name),
        rename_mod_build_file(project_root, old_project_name, new_project_name),
    ]);

    changeset.extend(api_reference_files.iter().map(|header| {
        replace_api_macro_in_header_file(project_root, header, old_project_name, new_project_name)
    }));

    changeset.extend(vec![
        rename_mod_header_file(project_root, old_project_name, new_project_name),
        replace_in_mod_source_file(project_root, old_project_name, new_project_name),
        rename_mod_source_file(project_root, old_project_name, new_project_name),
        rename_source_subfolder(project_root, old_project_name, new_project_name),
        update_redirects_in_engine_config(project_root, new_project_name),
        append_redirect_to_engine_config(project_root, old_project_name, new_project_name),
        add_game_name_to_engine_config(project_root, new_project_name),
        rename_project_root(project_root, new_project_name),
    ]);

    changeset
}

fn replace_in_project_descriptor(
    project_root: &Path,
    old_project_name: &str,
    new_project_name: &str,
) -> Change {
    Change::ReplaceInFile(ReplaceInFile::new(
        project_root
            .join(old_project_name)
            .with_extension("uproject"),
        old_project_name,
        new_project_name,
    ))
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

fn replace_in_exec_target_file(
    project_root: &Path,
    old_project_name: &str,
    new_project_name: &str,
) -> Change {
    Change::ReplaceInFile(ReplaceInFile::new(
        project_root
            .join("Source")
            .join(old_project_name)
            .with_extension("Target.cs"),
        old_project_name,
        new_project_name,
    ))
}

fn rename_exec_target_file(
    project_root: &Path,
    old_project_name: &str,
    new_project_name: &str,
) -> Change {
    Change::RenameFile(RenameFile::new(
        project_root
            .join("Source")
            .join(old_project_name)
            .with_extension("Target.cs"),
        project_root
            .join("Source")
            .join(new_project_name)
            .with_extension("Target.cs"),
    ))
}

fn replace_in_ed_target_file(
    project_root: &Path,
    old_project_name: &str,
    new_project_name: &str,
) -> Change {
    Change::ReplaceInFile(ReplaceInFile::new(
        project_root
            .join("Source")
            .join(format!("{}Editor", old_project_name))
            .with_extension("Target.cs"),
        old_project_name,
        new_project_name,
    ))
}

fn rename_ed_target_file(
    project_root: &Path,
    old_project_name: &str,
    new_project_name: &str,
) -> Change {
    Change::RenameFile(RenameFile::new(
        project_root
            .join("Source")
            .join(format!("{}Editor", old_project_name))
            .with_extension("Target.cs"),
        project_root
            .join("Source")
            .join(format!("{}Editor", new_project_name))
            .with_extension("Target.cs"),
    ))
}

fn replace_in_mod_build_file(
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

fn rename_mod_build_file(
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

fn rename_mod_header_file(
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

fn replace_in_mod_source_file(
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

fn rename_mod_source_file(
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

fn update_redirects_in_engine_config(project_root: &Path, new_project_name: &str) -> Change {
    Change::ReplaceInFile(ReplaceInFile::new(
        project_root.join("Config/DefaultEngine.ini"),
        r#"\(OldGameName="(?P<old>.+?)",\s*NewGameName=".+?"\)"#,
        format!(
            r#"(OldGameName="$old", NewGameName="/Script/{}")"#,
            new_project_name
        ),
    ))
}

fn append_redirect_to_engine_config(
    project_root: &Path,
    old_project_name: &str,
    new_project_name: &str,
) -> Change {
    Change::AppendIniEntry(AppendIniEntry::new(
        project_root.join("Config/DefaultEngine.ini"),
        "/Script/Engine.Engine",
        "+ActiveGameNameRedirects",
        format!(
            r#"(OldGameName="/Script/{}", NewGameName="/Script/{}")"#,
            old_project_name, new_project_name
        ),
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

fn rename_project_root(project_root: &Path, new_project_name: &str) -> Change {
    Change::RenameFile(RenameFile::new(
        &project_root,
        project_root.with_file_name(new_project_name),
    ))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::changes::*;

    use super::generate_code_changeset;

    #[test]
    fn code_changeset_is_correct() {
        let old_project_name = "Start";
        let new_project_name = "Finish";
        let project_root = "";
        let changeset = generate_code_changeset(
            old_project_name,
            new_project_name,
            project_root,
            vec![PathBuf::from("Source/Start/StartGameModeBase.h")],
        );
        let expected = vec![
            // Replace old name with new name in project descriptor
            Change::ReplaceInFile(ReplaceInFile::new(
                "Start.uproject",
                old_project_name,
                new_project_name,
            )),
            // Rename project descriptor
            Change::RenameFile(RenameFile::new("Start.uproject", "Finish.uproject")),
            // Replace old name with new name in executable target file
            Change::ReplaceInFile(ReplaceInFile::new(
                "Source/Start.Target.cs",
                old_project_name,
                new_project_name,
            )),
            // Rename executable target file
            Change::RenameFile(RenameFile::new(
                "Source/Start.Target.cs",
                "Source/Finish.Target.cs",
            )),
            // Replace old name with new name in editor target file
            Change::ReplaceInFile(ReplaceInFile::new(
                "Source/StartEditor.Target.cs",
                old_project_name,
                new_project_name,
            )),
            // Rename editor target file
            Change::RenameFile(RenameFile::new(
                "Source/StartEditor.Target.cs",
                "Source/FinishEditor.Target.cs",
            )),
            // Replace old name with new name in game module build file
            Change::ReplaceInFile(ReplaceInFile::new(
                "Source/Start/Start.Build.cs",
                old_project_name,
                new_project_name,
            )),
            // Rename game module build file
            Change::RenameFile(RenameFile::new(
                "Source/Start/Start.Build.cs",
                "Source/Start/Finish.Build.cs",
            )),
            // Replace old name with new name api references in header files
            Change::ReplaceInFile(ReplaceInFile::new(
                "Source/Start/StartGameModeBase.h",
                "START_API",
                "FINISH_API",
            )),
            // Rename game module header file
            Change::RenameFile(RenameFile::new(
                "Source/Start/Start.h",
                "Source/Start/Finish.h",
            )),
            // Replace old name with new name api references in header files
            Change::ReplaceInFile(ReplaceInFile::new(
                "Source/Start/Start.cpp",
                old_project_name,
                new_project_name,
            )),
            // Rename game module source file
            Change::RenameFile(RenameFile::new(
                "Source/Start/Start.cpp",
                "Source/Start/Finish.cpp",
            )),
            // Rename source subfolder
            Change::RenameFile(RenameFile::new("Source/Start", "Source/Finish")),
            // Update existing redirect entries in ini file
            Change::ReplaceInFile(ReplaceInFile::new(
                "Config/DefaultEngine.ini",
                r#"\(OldGameName="(?P<old>.+?)",\s*NewGameName=".+?"\)"#,
                r#"(OldGameName="$old", NewGameName="/Script/Finish")"#,
            )),
            // Append redirect entry to ini file
            Change::AppendIniEntry(AppendIniEntry::new(
                "Config/DefaultEngine.ini",
                "/Script/Engine.Engine",
                "+ActiveGameNameRedirects",
                r#"(OldGameName="/Script/Start", NewGameName="/Script/Finish")"#,
            )),
            // Add Game Name entry to ini file
            Change::SetIniEntry(SetIniEntry::new(
                "Config/DefaultEngine.ini",
                "URL",
                "GameName",
                "Finish",
            )),
            // Rename project root
            Change::RenameFile(RenameFile::new("", "Finish")),
        ];

        assert_eq!(changeset, expected);
    }
}
