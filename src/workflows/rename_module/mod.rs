mod changeset;
mod context;

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use inquire::{validator::Validation, Confirm, CustomUserError, Select, Text};
use regex::Regex;
use walkdir::WalkDir;

use crate::{
    engine::Engine,
    presentation::log,
    unreal::{Module, ModuleType, Plugin},
};

use self::{changeset::generate_changeset, context::Context};

pub fn start_rename_module_workflow() -> Result<(), String> {
    let context = gather_context()?;
    let changeset = generate_changeset(&context);
    let backup_dir = create_backup_dir(&context.project_root)?;
    let mut engine = Engine::new();
    if let Err(err) = engine.execute(changeset, backup_dir) {
        log::error(&err);
        if user_confirms_revert() {
            engine.revert()?;
        }
        print_failure_message(&context);
        return Ok(());
    }
    print_success_message(&context);
    Ok(())
}

fn gather_context() -> Result<Context, String> {
    let project_root = get_project_root_from_user()?;
    let project_name = detect_project_name(&project_root)?;
    let project_plugins = detect_project_plugins(&project_root)?;
    let modules = detect_project_modules(&project_root)?
        .into_iter()
        .chain(detect_plugin_modules(&project_plugins)?)
        .collect::<Vec<Module>>();
    let project_targets = detect_project_targets(&project_root)?;
    let project_config_files = detect_project_config_files(&project_root)?;
    let target_module = get_target_module_from_user(&modules)?;
    let target_name = get_target_name_from_user(&modules)?;
    let implementing_source = find_implementing_source(&target_module.root);
    let headers_with_export_macro =
        find_headers_with_export_macro(&target_module.root, &target_module.name);

    Ok(Context {
        project_root,
        project_name,
        project_targets,
        project_config_files,
        modules,
        target_module,
        target_name,
        source_with_implement_macro: implementing_source,
        headers_with_export_macro,
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

fn detect_project_targets(project_root: &Path) -> Result<Vec<PathBuf>, String> {
    let source_dir = project_root.join("Source");
    assert!(source_dir.is_dir());
    Ok(fs::read_dir(source_dir)
        .map_err(|err| err.to_string())?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .to_str()
                .map(|str| str.ends_with(".Target.cs"))
                .unwrap_or(false)
        })
        .map(|entry| entry.path().to_owned())
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

fn detect_project_config_files(project_root: &Path) -> Result<Vec<PathBuf>, String> {
    let config_dir = project_root.join("Config");
    Ok(WalkDir::new(config_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "ini"))
        .map(|entry| entry.path().to_owned())
        .collect())
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

fn find_implementing_source(module_root: &Path) -> Option<PathBuf> {
    WalkDir::new(module_root)
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.path().to_owned())
        .filter(|path| path.is_file() && path.extension().map_or(false, |ext| ext == "cpp"))
        .find(|source| {
            fs::read_to_string(source).map_or(false, |content| content.contains("_MODULE"))
        })
}

fn find_headers_with_export_macro(module_root: &Path, module_name: &str) -> Vec<PathBuf> {
    WalkDir::new(module_root)
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.path().to_owned())
        .filter(|path| {
            fs::read_to_string(path).map_or(false, |content| {
                content.contains(&format!("{}_API", module_name.to_uppercase()))
            })
        })
        .collect()
}

fn create_backup_dir(project_root: &Path) -> Result<PathBuf, String> {
    let backup_dir = project_root.join(".renom/backup");
    fs::create_dir_all(&backup_dir).map_err(|err| err.to_string())?;
    Ok(backup_dir)
}

/// Request revert desired from the user.
fn user_confirms_revert() -> bool {
    Confirm::new("Looks like something went wrong. Should we revert the changes made so far?")
        .prompt()
        .unwrap_or(false)
}

fn print_success_message(context: &Context) {
    log::success(format!(
        "Successfully renamed module {} to {}.",
        context.target_module.name, context.target_name
    ));
}

fn print_failure_message(context: &Context) {
    log::error(format!(
        "Failed to rename module {} to {}.",
        context.target_module.name, context.target_name
    ));
}
