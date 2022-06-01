use std::path::{Path, PathBuf};

use crate::change::{AppendIniEntry, Change, RenameFile, ReplaceInFile, SetIniEntry};

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
/// - Replace old name with new name in DefaultEngine config file
/// - Append redirect entry to DefaultEngine config file
/// - Add a GameName entry under the URL section to the DefaultEngine.ini config file
/// - Replace old name with new name in config files
/// - Rename project root directory
/// @todo: Update the api referencers to do proper replace
/// @todo: Update redirects in DefaultEngine config file
pub fn generate_code_changeset(
    old_project_name: &str,
    new_project_name: &str,
    project_root: impl AsRef<Path>,
    api_reference_files: Vec<PathBuf>,
    old_config_files: Vec<PathBuf>,
) -> Vec<Change> {
    let mut changeset = vec![];

    let project_root: PathBuf = project_root.as_ref().into();

    changeset.push(Change::ReplaceInFile(ReplaceInFile::new(
        project_root
            .join(old_project_name)
            .with_extension("uproject"),
        old_project_name,
        new_project_name,
    )));

    changeset.push(Change::RenameFile(RenameFile::new(
        project_root
            .join(old_project_name)
            .with_extension("uproject"),
        project_root
            .join(new_project_name)
            .with_extension("uproject"),
    )));

    changeset.push(Change::ReplaceInFile(ReplaceInFile::new(
        project_root
            .join("Source")
            .join(old_project_name)
            .with_extension("Target.cs"),
        old_project_name,
        new_project_name,
    )));

    changeset.push(Change::RenameFile(RenameFile::new(
        project_root
            .join("Source")
            .join(old_project_name)
            .with_extension("Target.cs"),
        project_root
            .join("Source")
            .join(new_project_name)
            .with_extension("Target.cs"),
    )));

    changeset.push(Change::ReplaceInFile(ReplaceInFile::new(
        project_root
            .join("Source")
            .join(format!("{}Editor", old_project_name))
            .with_extension("Target.cs"),
        old_project_name,
        new_project_name,
    )));

    changeset.push(Change::RenameFile(RenameFile::new(
        project_root
            .join("Source")
            .join(format!("{}Editor", old_project_name))
            .with_extension("Target.cs"),
        project_root
            .join("Source")
            .join(format!("{}Editor", new_project_name))
            .with_extension("Target.cs"),
    )));

    // Replace old name with new name in game module build file
    changeset.push(Change::ReplaceInFile(ReplaceInFile::new(
        project_root
            .join("Source")
            .join(old_project_name)
            .join(old_project_name)
            .with_extension("Build.cs"),
        old_project_name,
        new_project_name,
    )));

    // Rename game module build file
    changeset.push(Change::RenameFile(RenameFile::new(
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
    )));

    // Replace old name with new name api references in header files
    for referencer in api_reference_files {
        changeset.push(Change::ReplaceInFile(ReplaceInFile::new(
            project_root.join(referencer),
            format!("{}_API", old_project_name.to_uppercase()),
            format!("{}_API", new_project_name.to_uppercase()),
        )));
    }

    // Rename game module header file
    changeset.push(Change::RenameFile(RenameFile::new(
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
    )));

    // Replace old name with new name api references in header files
    changeset.push(Change::ReplaceInFile(ReplaceInFile::new(
        project_root
            .join("Source")
            .join(old_project_name)
            .join(old_project_name)
            .with_extension("cpp"),
        old_project_name,
        new_project_name,
    )));

    // Rename game module source file
    changeset.push(Change::RenameFile(RenameFile::new(
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
    )));

    // Rename source subfolder
    changeset.push(Change::RenameFile(RenameFile::new(
        project_root.join("Source").join(old_project_name),
        project_root.join("Source").join(new_project_name),
    )));

    // Replace old project name with new project name in ini file
    // @todo: Make this regex based to hit only redirects
    changeset.push(Change::ReplaceInFile(ReplaceInFile::new(
        project_root.join("Config/DefaultEngine.ini"),
        old_project_name,
        new_project_name,
    )));

    // Append redirect entry to ini file
    changeset.push(Change::AppendIniEntry(AppendIniEntry::new(
        project_root.join("Config/DefaultEngine.ini"),
        "/Script/Engine.Engine",
        "+ActiveGameNameRedirects",
        format!(
            r#"(OldGameName="/Script/{}", NewGameName="/Script/{}")"#,
            old_project_name, new_project_name
        ),
    )));

    // Add Game Name entry to ini file
    changeset.push(Change::SetIniEntry(SetIniEntry::new(
        project_root.join("Config/DefaultEngine.ini"),
        "URL",
        "GameName",
        new_project_name,
    )));

    // Replace old name with new name in config files
    for reference in old_config_files {
        changeset.push(Change::ReplaceInFile(ReplaceInFile::new(
            project_root.join(reference),
            old_project_name,
            new_project_name,
        )));
    }

    changeset.push(Change::RenameFile(RenameFile::new(
        &project_root,
        project_root.with_file_name(new_project_name),
    )));

    changeset
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::change::*;

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
            vec![PathBuf::from("Config/DefaultGame.ini")],
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
            // Replace old name with new name in ini file
            Change::ReplaceInFile(ReplaceInFile::new(
                "Config/DefaultEngine.ini",
                old_project_name,
                new_project_name,
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
            // Replace old name with new name in config files
            Change::ReplaceInFile(ReplaceInFile::new(
                "Config/DefaultGame.ini",
                old_project_name,
                new_project_name,
            )),
            // Rename project root
            Change::RenameFile(RenameFile::new("", "Finish")),
        ];

        assert_eq!(changeset, expected);
    }
}
