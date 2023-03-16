use std::{ffi::OsStr, fs, path::PathBuf};

use inquire::{validator::Validation, CustomUserError, Select, Text};

use crate::{changesets::generate_module_changeset, engine::Engine};

struct RenameModuleContext {
    project_root: PathBuf,
    project_name: String,
    project_modules: Vec<String>,
    target_module: String,
    target_name: String,
}

pub fn start_rename_module_workflow() -> Result<(), String> {
    let context = gather_context()?;
    let changeset = generate_module_changeset(
        &context.target_module,
        &context.target_name,
        &context.project_root,
        &context.project_name,
    );
    let backup_dir = context.project_root.join(".renom/backup");
    fs::create_dir_all(&backup_dir).map_err(|err| err.to_string())?;
    let mut engine = Engine::new();
    if let Err(e) = engine.execute(changeset, backup_dir) {
        println!("Err: {}", e);
        engine.revert();
        return Err(e);
    }
    Ok(())
}

fn gather_context() -> Result<RenameModuleContext, String> {
    let project_root = get_project_root_from_user()?;
    let project_name = detect_project_name(&project_root)?;
    let project_modules = detect_project_modules(&project_root)?;
    let target_module = get_target_module_from_user(&project_modules)?;
    let target_name = get_target_name_from_user(&project_modules)?;
    Ok(RenameModuleContext {
        project_root,
        project_name,
        project_modules,
        target_module,
        target_name,
    })
}

fn get_project_root_from_user() -> Result<PathBuf, String> {
    Text::new("Project root directory path:")
        .with_validator(validate_project_root_is_dir)
        .with_validator(validate_project_root_contains_project_descriptor)
        .with_validator(validate_project_root_contains_source_dir)
        .prompt()
        .map(|project_root| PathBuf::from(project_root))
        .map_err(|err| err.to_string())
}

fn validate_project_root_is_dir(project_root: &str) -> Result<Validation, CustomUserError> {
    match PathBuf::from(project_root).is_dir() {
        true => Ok(Validation::Valid),
        false => {
            let error_message = "Provided path is not a directory";
            Ok(Validation::Invalid(error_message.into()))
        }
    }
}

fn validate_project_root_contains_project_descriptor(
    project_root: &str,
) -> Result<Validation, CustomUserError> {
    match fs::read_dir(project_root)?
        .filter_map(Result::ok)
        .filter_map(|entry| entry.path().extension().map(OsStr::to_owned))
        .any(|ext| ext == "uproject")
    {
        true => Ok(Validation::Valid),
        false => {
            let error_message = "Provided directory does not contain a .uproject file";
            Ok(Validation::Invalid(error_message.into()))
        }
    }
}

fn validate_project_root_contains_source_dir(
    project_root: &str,
) -> Result<Validation, CustomUserError> {
    match PathBuf::from(project_root).join("Source").is_dir() {
        true => Ok(Validation::Valid),
        false => {
            let error_message = "Provided directory does not contain a Source folder";
            Ok(Validation::Invalid(error_message.into()))
        }
    }
}

/// Detect the name of a project given the path to the project root directory.
/// Assumes that the directory exists and that it contains a project descriptor.
/// Returns an error in case of I/O issues.
fn detect_project_name(project_root: &PathBuf) -> Result<String, String> {
    assert!(project_root.is_dir());

    let project_descriptor = fs::read_dir(project_root)
        .map_err(|err| err.to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().map_or(false, |ext| ext == "uproject"))
        .next()
        .expect("project descriptor should exist");

    project_descriptor
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|name| name.to_owned())
        .ok_or("project name is not valid Unicode".into())
}

/// Detect all project modules in a project given the path to the project root
/// directory. Assumes that the Source folder exists. Returns an error in case
/// of I/O issues.
fn detect_project_modules(project_root: &PathBuf) -> Result<Vec<String>, String> {
    let source_dir = project_root.join("Source");
    assert!(source_dir.is_dir());

    Ok(fs::read_dir(source_dir)
        .map_err(|err| err.to_string())?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter(|entry| {
            entry
                .path()
                .join(entry.path().file_name().expect("filename should exist"))
                .with_extension("Build.cs")
                .is_file()
        })
        .filter_map(|entry| entry.file_name().to_str().map(str::to_owned))
        .collect())
}

fn get_target_module_from_user(project_modules: &[String]) -> Result<String, String> {
    Select::new("Choose a module:", project_modules.to_vec())
        .prompt()
        .map_err(|err| err.to_string())
}

fn get_target_name_from_user(project_modules: &[String]) -> Result<String, String> {
    let project_modules = project_modules.to_vec();
    Text::new("Provide a new name for the module:")
        .with_validator(validate_target_name_is_not_empty)
        .with_validator(validate_target_name_is_concise)
        .with_validator(move |input: &str| validate_target_name_is_unique(input, &project_modules))
        .prompt()
        .map_err(|err| err.to_string())
}

fn validate_target_name_is_not_empty(target_name: &str) -> Result<Validation, CustomUserError> {
    match !target_name.trim().is_empty() {
        true => Ok(Validation::Valid),
        false => {
            let error_message = "Target name must not be empty";
            Ok(Validation::Invalid(error_message.into()))
        }
    }
}

fn validate_target_name_is_concise(target_name: &str) -> Result<Validation, CustomUserError> {
    let target_name_max_len = 30;
    match target_name.len() <= target_name_max_len {
        true => Ok(Validation::Valid),
        false => {
            let error_message = format!(
                "Target name must not be longer than {} characters",
                target_name_max_len
            );
            Ok(Validation::Invalid(error_message.into()))
        }
    }
}

fn validate_target_name_is_unique(
    target_name: &str,
    project_modules: &[String],
) -> Result<Validation, CustomUserError> {
    match project_modules.iter().all(|module| module != target_name) {
        true => Ok(Validation::Valid),
        false => {
            let error_message = "Target name must not conflict with another module";
            Ok(Validation::Invalid(error_message.into()))
        }
    }
}
