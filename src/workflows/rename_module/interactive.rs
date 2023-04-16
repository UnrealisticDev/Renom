use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use inquire::{validator::Validation, CustomUserError, Select, Text};
use regex::Regex;
use walkdir::WalkDir;

use crate::unreal::{Module, ModuleType, Plugin};

use super::Params;

pub fn get_params_from_user() -> Result<Params, String> {
    let project_root = get_project_root_from_user()?;
    let project_plugins = detect_project_plugins(&project_root)?;
    let modules = detect_project_modules(&project_root)?
        .into_iter()
        .chain(detect_plugin_modules(&project_plugins)?)
        .collect::<Vec<Module>>();
    let target_module = get_target_module_from_user(&modules)?;
    let target_name = get_target_name_from_user(&modules)?;

    Ok(Params {
        project_root,
        module: target_module.name,
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

/// Detect all plugins in a project given the path to the project root
/// directory. Detects top-level plugins and nested plugins. Returns an error in
/// case of I/O issues.
fn detect_project_plugins(project_root: &PathBuf) -> Result<Vec<Plugin>, String> {
    let plugins_dir = project_root.join("Plugins");
    Ok(WalkDir::new(plugins_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map_or(false, |ext| ext == "uplugin")
        })
        .map(|entry| Plugin {
            root: entry.path().parent().unwrap().to_owned(),
            name: entry
                .path()
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned(),
        })
        .collect())
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
            r#type: ModuleType::Project,
            plugin: None,
        })
        .collect())
}

/// Detect all plugin modules in a project given the list of project plugins.
/// Detects top-level modules and nested modules. Returns an error in case of
/// I/O issues.
fn detect_plugin_modules(project_plugins: &[Plugin]) -> Result<Vec<Module>, String> {
    Ok(project_plugins
        .iter()
        .flat_map(|plugin| {
            WalkDir::new(&plugin.root)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|entry| {
                    entry.path().is_dir() && dir_contains_module_descriptor(entry.path())
                })
                .map(move |entry| Module {
                    root: entry.path().to_owned(),
                    name: get_dir_name(&entry.path()),
                    r#type: ModuleType::Plugin,
                    plugin: Some(plugin.clone()),
                })
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

fn get_target_module_from_user(modules: &[Module]) -> Result<Module, String> {
    Select::new("Choose a module:", modules.to_vec())
        .prompt()
        .map_err(|err| err.to_string())
}

fn get_target_name_from_user(modules: &[Module]) -> Result<String, String> {
    let modules = modules.to_vec();
    Text::new("Provide a new name for the module:")
        .with_validator(validate_target_name_is_not_empty)
        .with_validator(validate_target_name_is_concise)
        .with_validator(move |input: &str| validate_target_name_is_unique(input, &modules))
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
    modules: &[Module],
) -> Result<Validation, CustomUserError> {
    match modules.iter().all(|module| module.name != target_name) {
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
