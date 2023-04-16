use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::workflows::{rename_module, rename_plugin, rename_project, rename_target};

#[derive(Parser)]
#[command(author, version, about, arg_required_else_help(true))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(PartialEq, Debug, Subcommand)]
pub enum Command {
    /// Rename an Unreal Engine project
    RenameProject(RenameProject),
    /// Rename an Unreal Engine project plugin
    RenamePlugin(RenamePlugin),
    /// Rename an Unreal Engine project target
    RenameTarget(RenameTarget),
    /// Rename an Unreal Engine project module
    RenameModule(RenameModule),
    /// Start an interactive session
    Wizard,
}

#[derive(PartialEq, Debug, Parser)]
pub struct RenameProject {
    /// Path to the project to rename
    #[arg(long)]
    project: PathBuf,
    /// New name for the project
    #[arg(long)]
    new_name: String,
}

impl From<RenameProject> for rename_project::Params {
    fn from(params: RenameProject) -> Self {
        Self {
            project_root: params.project,
            new_name: params.new_name,
        }
    }
}

#[derive(PartialEq, Debug, Parser)]
pub struct RenamePlugin {
    /// Path to the project that the plugin is part of
    #[arg(long)]
    project: PathBuf,
    /// Plugin in the project to rename
    #[arg(long)]
    plugin: String,
    /// New name for the plugin
    #[arg(long)]
    new_name: String,
}

impl From<RenamePlugin> for rename_plugin::Params {
    fn from(params: RenamePlugin) -> Self {
        Self {
            project_root: params.project,
            plugin: params.plugin,
            new_name: params.new_name,
        }
    }
}

#[derive(PartialEq, Debug, Parser)]
pub struct RenameTarget {
    /// Path to the project that the target is part of
    #[arg(long)]
    project: PathBuf,
    /// Target in the project to rename
    #[arg(long)]
    target: String,
    /// New name for the target
    #[arg(long)]
    new_name: String,
}

impl From<RenameTarget> for rename_target::Params {
    fn from(params: RenameTarget) -> Self {
        Self {
            project_root: params.project,
            target: params.target,
            new_name: params.new_name,
        }
    }
}

#[derive(PartialEq, Debug, Parser)]
pub struct RenameModule {
    /// Path to the project that the module is part of
    #[arg(long)]
    project: PathBuf,
    /// Module in the project to rename
    #[arg(long)]
    module: String,
    /// New name for the module
    #[arg(long)]
    new_name: String,
}

impl From<RenameModule> for rename_module::Params {
    fn from(params: RenameModule) -> Self {
        Self {
            project_root: params.project,
            module: params.module,
            new_name: params.new_name,
        }
    }
}
