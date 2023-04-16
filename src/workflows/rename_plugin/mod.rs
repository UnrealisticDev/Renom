mod changeset;
mod interactive;

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use regex::Regex;
use walkdir::WalkDir;

use crate::{engine::Engine, presentation::log, unreal::Plugin};

use self::{changeset::generate_changeset, interactive::get_params_from_user};

/// Params needed to rename an Unreal Engine plugin.
pub struct Params {
    /// The root of the project.
    pub project_root: PathBuf,
    /// The specific plugin to rename.
    pub plugin: String,
    /// The new name for the plugin.
    pub new_name: String,
}

/// Context needed to rename an Unreal Engine plugin.
pub struct Context {
    /// The root of the project.
    pub project_root: PathBuf,
    /// The name of the project.
    pub project_name: String,
    /// Plugins for the project.
    pub project_plugins: Vec<Plugin>,
    /// The specific plugin to rename.
    pub plugin: Plugin,
    /// The new name for the plugin.
    pub new_name: String,
}

/// Rename an Unreal Engine plugin interactively, soliciting input parameters
/// from the user with validation and guided selection.
pub fn rename_plugin_interactive() -> Result<(), String> {
    let params = get_params_from_user()?;
    rename_plugin(params)
}

/// Rename an Unreal Engine plugin.
pub fn rename_plugin(params: Params) -> Result<(), String> {
    validate_params(&params)?;
    let context = gather_context(&params)?;
    let changeset = generate_changeset(&context);
    let backup_dir = create_backup_dir(&context.project_root)?;
    let mut engine = Engine::new();
    if let Err(e) = engine.execute(changeset, backup_dir) {
        log::error(&e);
        engine.revert()?;
        print_failure_message(&context);
        return Ok(());
    }

    print_success_message(&context);
    Ok(())
}

fn validate_params(params: &Params) -> Result<(), String> {
    validate_project_root_is_dir(&params.project_root)?;
    validate_project_root_contains_project_descriptor(&params.project_root)?;
    validate_project_root_contains_source_dir(&params.project_root)?;
    let plugins = detect_project_plugins(&params.project_root)?;
    validate_plugin_exists(&params.plugin, &plugins)?;
    validate_new_name_is_not_empty(&params.new_name)?;
    validate_new_name_is_concise(&params.new_name)?;
    validate_new_name_is_unique(&params.new_name, &plugins)?;
    validate_new_name_is_valid_identifier(&params.new_name)?;
    Ok(())
}

fn validate_project_root_is_dir(project_root: &Path) -> Result<(), String> {
    match project_root.is_dir() {
        true => Ok(()),
        false => Err("project root must be a directory".into()),
    }
}

fn validate_project_root_contains_project_descriptor(project_root: &Path) -> Result<(), String> {
    match fs::read_dir(project_root)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .filter_map(|entry| entry.path().extension().map(OsStr::to_owned))
        .any(|ext| ext == "uproject")
    {
        true => Ok(()),
        false => Err("project root must contain a project descriptor".into()),
    }
}

fn validate_project_root_contains_source_dir(project_root: &Path) -> Result<(), String> {
    match project_root.join("Source").is_dir() {
        true => Ok(()),
        false => Err("project root must contain a Source folder".into()),
    }
}

fn validate_plugin_exists(plugin: &str, plugins: &[Plugin]) -> Result<(), String> {
    match plugins.iter().any(|other| other.name == plugin) {
        true => Ok(()),
        false => Err("plugin must be part of project".into()),
    }
}

fn validate_new_name_is_not_empty(new_name: &str) -> Result<(), String> {
    match !new_name.trim().is_empty() {
        true => Ok(()),
        false => Err("new name must not be empty".into()),
    }
}

fn validate_new_name_is_concise(new_name: &str) -> Result<(), String> {
    let new_name_max_len = 30;
    match new_name.len() <= new_name_max_len {
        true => Ok(()),
        false => {
            let error_message = format!(
                "new name must not be longer than {} characters",
                new_name_max_len
            );
            Err(error_message)
        }
    }
}

fn validate_new_name_is_unique(new_name: &str, plugins: &[Plugin]) -> Result<(), String> {
    match plugins.iter().all(|plugin| plugin.name != new_name) {
        true => Ok(()),
        false => {
            let error_message = "new name must not conflict with another plugin";
            Err(error_message.into())
        }
    }
}

fn validate_new_name_is_valid_identifier(new_name: &str) -> Result<(), String> {
    let identifier_regex = Regex::new("^[_[[:alnum:]]]*$").expect("regex should be valid");
    match identifier_regex.is_match(new_name) {
        true => Ok(()),
        false => {
            let error_message =
                "new name must be comprised of alphanumeric characters and underscores only";
            Err(error_message.into())
        }
    }
}

fn gather_context(params: &Params) -> Result<Context, String> {
    let project_name = detect_project_name(&params.project_root)?;
    let project_plugins = detect_project_plugins(&params.project_root)?;
    let plugin = project_plugins
        .iter()
        .find(|plugin| plugin.name == params.plugin)
        .unwrap()
        .clone();

    Ok(Context {
        project_root: params.project_root.clone(),
        project_name,
        project_plugins,
        plugin,
        new_name: params.new_name.clone(),
    })
}

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

fn create_backup_dir(project_root: &Path) -> Result<PathBuf, String> {
    let backup_dir = project_root.join(".renom/backup");
    fs::create_dir_all(&backup_dir).map_err(|err| err.to_string())?;
    Ok(backup_dir)
}

fn print_success_message(context: &Context) {
    log::success(format!(
        "Successfully renamed plugin {} to {}.",
        context.plugin.name, context.new_name
    ));
}

fn print_failure_message(context: &Context) {
    log::error(format!(
        "Failed to rename plugin {} to {}.",
        context.plugin.name, context.new_name
    ));
}
