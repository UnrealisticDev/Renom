use clap::Parser;
use renom::{
    cli::{
        Cli,
        Command::{RenameModule, RenamePlugin, RenameProject, RenameTarget, Wizard},
    },
    presentation::log,
    wizard::start_interactive_dialogue,
    workflows::{rename_module, rename_plugin, rename_project, rename_target},
};
mod crash;

fn main() {
    crash::init_crash_reporter();

    let cli = Cli::parse();
    match cli.command {
        None => { /* noop, clap will handle top-level help and version */ }
        Some(command) => {
            if let Err(e) = match command {
                RenameProject(params) => rename_project(params.into()),
                RenamePlugin(params) => rename_plugin(params.into()),
                RenameTarget(params) => rename_target(params.into()),
                RenameModule(params) => rename_module(params.into()),
                Wizard => {
                    start_interactive_dialogue();
                    Ok(())
                }
            } {
                log::error(e);
            }
        }
    };
}
