use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use inquire::{validator::Validation, CustomUserError, Select, Text};
use regex::Regex;
use walkdir::WalkDir;

use crate::{changesets::generate_module_changeset, engine::Engine, unreal::Module};

struct RenameModuleContext {
    project_root: PathBuf,
    project_name: String,
    project_modules: Vec<Module>,
    target_module: Module,
    target_name: String,
}

pub fn start_rename_module_workflow() -> Result<(), String> {
    let context = gather_context()?;
    let changeset = generate_module_changeset(
        &context.target_module.name,
        &context.target_module.root,
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
/// directory. Detects top-level modules and nested modules. Assumes that the
/// Source folder exists. Returns an error in case of I/O issues.
fn detect_project_modules(project_root: &PathBuf) -> Result<Vec<Module>, String> {
    let source_dir = project_root.join("Source");
    assert!(source_dir.is_dir());
    Ok(WalkDir::new(source_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir() && dir_contains_module_descriptor(entry.path()))
        .map(|entry| Module {
            root: entry.path().to_owned(),
            name: get_dir_name(&entry.path()),
        })
        .collect())
}

fn dir_contains_module_descriptor(dir: &Path) -> bool {
    assert!(dir.is_dir());
    let dir_name = dir.file_name().expect("directory name should exist");
    dir.join(dir_name).with_extension("Build.cs").is_file()
}

fn get_dir_name(dir: &Path) -> String {
    dir.file_name()
        .expect("directory name should exist")
        .to_str()
        .expect("name should be valid Unicode")
        .to_string()
}

fn get_target_module_from_user(project_modules: &[Module]) -> Result<Module, String> {
    Select::new("Choose a module:", project_modules.to_vec())
        .prompt()
        .map_err(|err| err.to_string())
}

fn get_target_name_from_user(project_modules: &[Module]) -> Result<String, String> {
    let project_modules = project_modules.to_vec();
    Text::new("Provide a new name for the module:")
        .with_validator(validate_target_name_is_not_empty)
        .with_validator(validate_target_name_is_concise)
        .with_validator(move |input: &str| validate_target_name_is_unique(input, &project_modules))
        .with_validator(validate_target_name_is_valid_identifier)
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
    project_modules: &[Module],
) -> Result<Validation, CustomUserError> {
    match project_modules
        .iter()
        .all(|module| module.name != target_name)
    {
        true => Ok(Validation::Valid),
        false => {
            let error_message = "Target name must not conflict with another module";
            Ok(Validation::Invalid(error_message.into()))
        }
    }
}

fn validate_target_name_is_valid_identifier(
    target_name: &str,
) -> Result<Validation, CustomUserError> {
    let identifier_regex = Regex::new("^[_[[:alnum:]]]*$").expect("regex should be valid");
    match identifier_regex.is_match(target_name) {
        true => Ok(Validation::Valid),
        false => {
            let error_message =
                "Target name must be comprised of alphanumeric characters and underscores only";
            Ok(Validation::Invalid(error_message.into()))
        }
    }
}
