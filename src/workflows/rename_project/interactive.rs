use std::{ffi::OsStr, fs, path::PathBuf};

use inquire::{validator::Validation, CustomUserError, Text};
use regex::Regex;

use super::Params;

pub fn get_params_from_user() -> Result<Params, String> {
    let project_root = get_project_root_from_user()?;
    let target_name = get_target_name_from_user()?;
    Ok(Params {
        project_root,
        new_name: target_name,
    })
}

fn get_project_root_from_user() -> Result<PathBuf, String> {
    Text::new("Project root directory path:")
        .with_validator(validate_project_root_is_dir)
        .with_validator(validate_project_root_contains_project_descriptor)
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

fn get_target_name_from_user() -> Result<String, String> {
    Text::new("Provide a new name for the project:")
        .with_validator(validate_target_name_is_not_empty)
        .with_validator(validate_target_name_is_concise)
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
    let target_name_max_len = 20;
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
