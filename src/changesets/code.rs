use std::{fs, io::stdin, path::Path};

use crate::{
    changes::{AppendIniEntry, Change, RenameFile, ReplaceInFile, SetIniEntry},
    presentation::log,
};

use super::{generate_module_changeset, generate_target_changeset};

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
/// - Add a ProjectName entry under the GeneralProjectSettings section to the DefaultGame.ini config file
/// - Rename project root directory
pub fn generate_code_changeset(
    old_project_name: &str,
    new_project_name: &str,
    project_root: impl AsRef<Path>,
) -> Vec<Change> {
    let project_root = project_root.as_ref();
    let mut changeset = vec![];

    // do modules first, we can avoid having to
    // track changes that might happen to target files
    // @todo: introduce opt-out mechanism
    // do not need to rename all modules
    find_module_names(project_root)
        .iter()
        .for_each(|old_module_name| {
            log::basic(format!("Found project module named {}.", old_module_name));
            log::prompt("Target module name");
            let new_module_name = request_final_module_name();
            changeset.extend(generate_module_changeset(
                old_module_name,
                project_root.join("Source").join(old_module_name),
                &new_module_name,
                project_root,
                old_project_name,
            ))
        });

    // @todo: introduce opt-out mechanism
    // do not need to rename all targets
    // @todo: surface read errors
    find_target_file_names(project_root)
        .iter()
        .for_each(|old_target_name| {
            log::basic(format!("Found project target named {}.", old_target_name));
            log::prompt("Target final name");
            let new_target_name = request_final_target_name();
            changeset.extend(generate_target_changeset(
                old_target_name,
                &new_target_name,
                project_root,
            ))
        });

    changeset.extend(vec![
        update_redirects_in_engine_config(project_root, new_project_name),
        append_redirect_to_engine_config(project_root, old_project_name, new_project_name),
        add_game_name_to_engine_config(project_root, new_project_name),
        add_project_name_to_game_config(project_root, new_project_name),
        replace_in_project_descriptor(project_root, old_project_name, new_project_name),
        rename_project_descriptor(project_root, old_project_name, new_project_name),
        rename_project_root(project_root, new_project_name),
    ]);

    changeset
}

fn request_final_target_name() -> String {
    let mut buffer = String::new();
    stdin()
        .read_line(&mut buffer)
        .map(|_| String::from(buffer.trim()))
        .map_err(|err| err.to_string())
        .unwrap()
}

fn find_module_names(project_root: &Path) -> Vec<String> {
    fs::read_dir(project_root.join("Source"))
        .expect("could not read source dir")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|filename| filename.to_owned())
        })
        .collect()
}

fn request_final_module_name() -> String {
    let mut buffer = String::new();
    stdin()
        .read_line(&mut buffer)
        .map(|_| String::from(buffer.trim()))
        .map_err(|err| err.to_string())
        .unwrap()
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
    use crate::changes::*;

    use super::generate_code_changeset;

    #[test]
    fn code_changeset_is_correct() {
        let old_project_name = "Start";
        let new_project_name = "Finish";
        let project_root = "";
        let changeset = generate_code_changeset(old_project_name, new_project_name, project_root);
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
