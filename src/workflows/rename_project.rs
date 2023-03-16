use std::{
    fs,
    io::{stdin, Read},
    path::{Path, PathBuf},
};

use crate::{
    changesets::{generate_blueprint_changeset, generate_code_changeset},
    engine::Engine,
    presentation::log,
};

enum ProjectType {
    Blueprint,
    Code,
}

pub fn start_rename_project_workflow() -> Result<(), String> {
    log::header("Project Details");
    log::basic("Tell us a little about your project.");

    log::prompt("Project root directory path");
    let project_root = request_project_root()?;

    let original_name = infer_original_project_name(&project_root)?;
    log::basic(format!("Project original name: {}", &original_name));

    log::prompt("Project final name");
    let final_name = request_final_project_name(&original_name)?;

    let project_type = detect_project_type(&project_root);
    match project_type {
        ProjectType::Blueprint => log::basic("Blueprint project detected."),
        ProjectType::Code => log::basic("Code project detected."),
    }

    log::header("Staging");
    let backup_dir = create_backup_dir(&project_root)?;
    log::basic(format!(
        "Created backup directory at {}",
        backup_dir.to_str().unwrap()
    ));

    let changeset = match project_type {
        ProjectType::Blueprint => {
            generate_blueprint_changeset(&original_name, &final_name, &project_root)
        }
        ProjectType::Code => generate_code_changeset(&original_name, &final_name, &project_root),
    };

    log::header("Application");
    let mut engine = Engine::new();
    match engine.execute(changeset, &backup_dir) {
        Ok(_) => {
            log::header("Cleanup");
            match project_type {
                ProjectType::Blueprint => {
                    log::basic("Nothing to clean up for Blueprint project.");
                }
                ProjectType::Code => {
                    log::basic("Though not strictly necessary, it is a good idea to clean up outdated Saved, Intermediate, and Binaries folders.\nShall we go ahead and do so for you?");
                    log::prompt("[Y]es/[N]o");
                    if request_cleanup() {
                        cleanup(&project_root.with_file_name(&final_name))?;
                    } else {
                        log::basic("Cleanup skipped.");
                    }
                }
            }

            log::header("Success");
            log::basic("Project successfully renamed.");
        }
        Err(err) => {
            log::error(err);
            log::header("Recovery");
            log::basic("Looks like things did not work out as planned. Would you like to revert the changes made so far?");
            log::prompt("[Y]es/[N]o");
            if request_recover() {
                engine.revert()?;
            } else {
                log::basic("Recovery skipped.");
            }
        }
    }

    log::newline();
    log::prompt("Press Enter to exit.");
    let _ = stdin().read(&mut [0u8]);

    Ok(())
}

/// Request the project root directory from the user.
fn request_project_root() -> Result<PathBuf, String> {
    let mut buffer = String::new();
    let root = stdin()
        .read_line(&mut buffer)
        .map(|_| PathBuf::from(buffer.trim()))
        .map_err(|err| err.to_string())?;
    root.is_dir()
        .then(|| root)
        .ok_or_else(|| "Provided path was not a directory".to_owned())
}

/// Infer the project's original name from the project root.
fn infer_original_project_name(project_root: &Path) -> Result<String, String> {
    let project_descriptor = fs::read_dir(project_root)
        .map_err(|err| err.to_string())?
        .find_map(|entry| {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "uproject" {
                        return Some(path);
                    }
                }
            }
            None
        });

    project_descriptor
        .map(|path| path.file_stem().unwrap().to_str().unwrap().to_owned())
        .ok_or_else(|| "Failed to find .uproject file in project root".to_owned())
}

/// Request final project name from the user.
fn request_final_project_name(original_name: &str) -> Result<String, String> {
    let mut buffer = String::new();
    let final_name = stdin()
        .read_line(&mut buffer)
        .map(|_| String::from(buffer.trim()))
        .map_err(|err| err.to_string())?;

    if final_name.len() > 20 {
        return Err("Name is too long.".to_owned());
    }

    if final_name == original_name {
        return Err("Final name is identical to original name.".to_owned());
    }

    Ok(final_name)
}

/// Detect project type (Blueprint or C++) based on existence of
/// *Source* directory.
fn detect_project_type(project_root: &Path) -> ProjectType {
    if project_root.join("Source").is_dir() {
        ProjectType::Code
    } else {
        ProjectType::Blueprint
    }
}

/// Create a directory to store backup files in
fn create_backup_dir(project_root: &Path) -> Result<PathBuf, String> {
    let backup_dir = project_root.join(".renom/backup");
    fs::create_dir_all(&backup_dir).map_err(|err| err.to_string())?;
    Ok(backup_dir)
}

/// Request recover desired from the user.
fn request_recover() -> bool {
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).unwrap();
    matches!(buffer.trim(), "y" | "Y" | "yes" | "Yes")
}

/// Request cleanup desired from the user.
fn request_cleanup() -> bool {
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).unwrap();
    matches!(buffer.trim(), "y" | "Y" | "yes" | "Yes")
}

/// Cleanup *Saved*, *Intermediate*, and *Binaries* directories.
fn cleanup(project_root: &Path) -> Result<(), String> {
    log::basic("Deleting Saved directory.");
    let saved_dir = project_root.join("Saved");
    if saved_dir.is_dir() {
        fs::remove_dir_all(saved_dir).map_err(|err| err.to_string())?;
    } else {
        log::basic("Does not exist. Skipped.");
    }

    log::basic("Deleting Intermediate directory.");
    let intermediate_dir = project_root.join("Intermediate");
    if intermediate_dir.is_dir() {
        fs::remove_dir_all(intermediate_dir).map_err(|err| err.to_string())?;
    } else {
        log::basic("Does not exist. Skipped.");
    }

    log::basic("Deleting Binaries directory.");
    let binaries_dir = project_root.join("Binaries");
    if binaries_dir.is_dir() {
        fs::remove_dir_all(binaries_dir).map_err(|err| err.to_string())?;
    } else {
        log::basic("Does not exist. Skipped.");
    }

    Ok(())
}
