use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use inquire::{validator::Validation, CustomUserError, Select, Text};
use regex::Regex;

use crate::unreal::Target;

use super::Params;

pub fn get_params_from_user() -> Result<Params, String> {
    let project_root = get_project_root_from_user()?;
    let project_targets = detect_project_targets(&project_root)?;
    let target_target = get_target_target_from_user(&project_targets)?;
    let target_name = get_target_name_from_user(&project_targets)?;
    Ok(Params {
        project_root,
        target: target_target.name,
        new_name: target_name,
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

fn detect_project_targets(project_root: &Path) -> Result<Vec<Target>, String> {
    let source_dir = project_root.join("Source");
    assert!(source_dir.is_dir());
    Ok(fs::read_dir(&source_dir)
        .map_err(|err| err.to_string())?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            entry
                .path()
                .file_name()
                .and_then(|file_name| file_name.to_str())
                .and_then(|str| str.strip_suffix(".Target.cs"))
                .map(|str| str.to_owned())
        })
        .map(|target_name| Target {
            name: target_name.clone(),
            path: source_dir.join(target_name).with_extension("Target.cs"),
        })
        .collect())
}

fn get_target_target_from_user(targets: &[Target]) -> Result<Target, String> {
    Select::new("Choose a target:", targets.to_vec())
        .prompt()
        .map_err(|err| err.to_string())
}

fn get_target_name_from_user(targets: &[Target]) -> Result<String, String> {
    let targets = targets.to_vec();
    Text::new("Provide a new name for the target:")
        .with_validator(validate_target_name_is_not_empty)
        .with_validator(validate_target_name_is_concise)
        .with_validator(move |input: &str| validate_target_name_is_unique(input, &targets))
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
    targets: &[Target],
) -> Result<Validation, CustomUserError> {
    match targets.iter().all(|target| target.name != target_name) {
        true => Ok(Validation::Valid),
        false => {
            let error_message = "Target name must not conflict with another target";
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
