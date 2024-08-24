mod changeset;
mod interactive;

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use regex::Regex;

use crate::{engine::Engine, presentation::log};

use self::{changeset::generate_changeset, interactive::get_params_from_user};

/// Params needed to rename an Unreal Engine project.
pub struct Params {
    /// The root of the project.
    pub project_root: PathBuf,
    /// The new name for the project.
    pub new_name: String,
}

/// Context needed to rename an Unreal Engine project.
pub struct Context {
    /// The root of the project.
    pub project_root: PathBuf,
    /// The name of the project.
    pub project_name: String,
    /// The new name for the project.
    pub new_name: String,
}

/// Rename an Unreal Engine project interactively, soliciting input parameters
/// from the user with validation and guided selection.
pub fn rename_project_interactive() -> Result<(), String> {
    let params = get_params_from_user()?;
    rename_project(params)
}

/// Rename an Unreal Engine project.
pub fn rename_project(params: Params) -> Result<(), String> {
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
    validate_project_root_is_not_special(&params.project_root)?;
    validate_project_root_is_dir(&params.project_root)?;
    validate_project_root_contains_project_descriptor(&params.project_root)?;
    let project_name = detect_project_name(&params.project_root)?;
    validate_new_name_is_not_empty(&params.new_name)?;
    validate_new_name_is_novel(&project_name, &params.new_name)?;
    validate_new_name_is_concise(&params.new_name)?;
    validate_new_name_is_valid_identifier(&params.new_name)?;
    Ok(())
}

fn validate_project_root_is_not_special(project_root: &Path) -> Result<(), String> {
    match project_root {
        path if path == Path::new(".") => Err("project root cannot be '.'".into()),
        path if path == Path::new("..") => Err("project root cannot be '..'".into()),
        _ => Ok(()),
    }
}

fn validate_project_root_is_dir(project_root: &Path) -> Result<(), String> {
    match project_root.is_dir() {
        true => Ok(()),
        false => Err("project root must be a directory".into()),
    }
}

fn validate_project_root_contains_project_descriptor(project_root: &Path) -> Result<(), String> {
    match fs::read_dir(&project_root)
        .map_err(|err| err.to_string())?
        .filter_map(Result::ok)
        .filter_map(|entry| entry.path().extension().map(OsStr::to_owned))
        .any(|ext| ext == "uproject")
    {
        true => Ok(()),
        false => Err("project root must contain a project descriptor".into()),
    }
}

fn validate_new_name_is_novel(old_name: &str, new_name: &str) -> Result<(), String> {
    match old_name != new_name {
        true => Ok(()),
        false => Err("new name must be different than current name".into()),
    }
}

fn validate_new_name_is_not_empty(new_name: &str) -> Result<(), String> {
    match !new_name.trim().is_empty() {
        true => Ok(()),
        false => Err("new name must not be empty".into()),
    }
}

fn validate_new_name_is_concise(new_name: &str) -> Result<(), String> {
    let new_name_max_len = 20;
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
    let project_name = detect_project_name(&PathBuf::from(&params.project_root))?;
    Ok(Context {
        project_root: params.project_root.clone(),
        project_name,
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

/// Create a directory to store backup files in
fn create_backup_dir(project_root: &Path) -> Result<PathBuf, String> {
    let backup_dir = project_root.join(".renom/backup");
    fs::create_dir_all(&backup_dir).map_err(|err| err.to_string())?;
    Ok(backup_dir)
}

fn print_success_message(context: &Context) {
    log::success(format!(
        "Successfully renamed project {} to {}.",
        context.project_name, context.new_name
    ));
}

fn print_failure_message(context: &Context) {
    log::error(format!(
        "Failed to rename project {} to {}.",
        context.project_name, context.new_name
    ));
}
