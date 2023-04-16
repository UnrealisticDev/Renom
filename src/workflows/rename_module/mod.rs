mod changeset;
mod interactive;

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use regex::Regex;
use walkdir::WalkDir;

use crate::{
    engine::Engine,
    presentation::log,
    unreal::{Module, ModuleType, Plugin},
};

use self::{changeset::generate_changeset, interactive::get_params_from_user};

/// Params needed to rename an Unreal Engine module.
pub struct Params {
    /// The root of the project.
    pub project_root: PathBuf,
    /// The specific module to rename.
    pub module: String,
    /// The new name for the module.
    pub new_name: String,
}

/// Context needed to rename an Unreal Engine module.
pub struct Context {
    /// The root of the project that the module is part of.
    pub project_root: PathBuf,
    /// The name of the project.
    pub project_name: String,
    /// Build targets for the project that the module is part of.
    pub project_targets: Vec<PathBuf>,
    /// Config files for the project.
    pub project_config_files: Vec<PathBuf>,
    /// Code modules in the project.
    pub modules: Vec<Module>,
    /// The specific module to rename.
    pub module: Module,
    /// The new name for the module.
    pub new_name: String,
    /// The source file that includes the module implement macro.
    pub source_with_implement_macro: Option<PathBuf>,
    /// Header files that include the module export macro.
    pub headers_with_export_macro: Vec<PathBuf>,
}

/// Rename an Unreal Engine module interactively, soliciting input parameters
/// from the user with validation and guided selection.
pub fn rename_module_interactive() -> Result<(), String> {
    let params = get_params_from_user()?;
    rename_module(params)
}

/// Rename an Unreal Engine module.
pub fn rename_module(params: Params) -> Result<(), String> {
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
    let project_plugins = detect_project_plugins(&params.project_root)?;
    let modules = detect_project_modules(&params.project_root)?
        .into_iter()
        .chain(detect_plugin_modules(&project_plugins)?)
        .collect::<Vec<Module>>();
    validate_module_exists(&params.module, &modules)?;
    validate_new_name_is_not_empty(&params.new_name)?;
    validate_new_name_is_concise(&params.new_name)?;
    validate_new_name_is_unique(&params.new_name, &modules)?;
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

fn validate_module_exists(module: &str, modules: &[Module]) -> Result<(), String> {
    match modules.iter().any(|other| other.name == module) {
        true => Ok(()),
        false => Err("module must be part of project".into()),
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

fn validate_new_name_is_unique(new_name: &str, modules: &[Module]) -> Result<(), String> {
    match modules.iter().all(|module| module.name != new_name) {
        true => Ok(()),
        false => {
            let error_message = "new name must not conflict with another module";
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

fn gather_context(params: &Params) -> Result<Context, String> {
    let project_root = params.project_root.clone();
    let project_name = detect_project_name(&project_root)?;
    let project_plugins = detect_project_plugins(&project_root)?;
    let modules = detect_project_modules(&project_root)?
        .into_iter()
        .chain(detect_plugin_modules(&project_plugins)?)
        .collect::<Vec<Module>>();
    let project_targets = detect_project_targets(&project_root)?;
    let project_config_files = detect_project_config_files(&project_root)?;
    let target_module = modules
        .iter()
        .find(|module| module.name == params.module)
        .unwrap()
        .clone();
    let implementing_source = find_implementing_source(&target_module.root);
    let headers_with_export_macro =
        find_headers_with_export_macro(&target_module.root, &target_module.name);

    Ok(Context {
        project_root,
        project_name,
        project_targets,
        project_config_files,
        modules,
        module: target_module,
        new_name: params.new_name.clone(),
        source_with_implement_macro: implementing_source,
        headers_with_export_macro,
    })
}

fn create_backup_dir(project_root: &Path) -> Result<PathBuf, String> {
    let backup_dir = project_root.join(".renom/backup");
    fs::create_dir_all(&backup_dir).map_err(|err| err.to_string())?;
    Ok(backup_dir)
}

fn print_success_message(context: &Context) {
    log::success(format!(
        "Successfully renamed module {} to {}.",
        context.module.name, context.new_name
    ));
}

fn print_failure_message(context: &Context) {
    log::error(format!(
        "Failed to rename module {} to {}.",
        context.module.name, context.new_name
    ));
}
