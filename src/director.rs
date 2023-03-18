use inquire::{Confirm, Select};

use crate::{
    presentation::log,
    workflows::{start_rename_module_workflow, start_rename_project_workflow, Workflow},
};

/// Takes a result and returns its inner
/// value if it is ok. In the case of error,
/// logs the error and returns from the function.
macro_rules! ok_or_quit {
    ( $e:expr ) => {
        match $e {
            Ok(t) => t,
            Err(e) => {
                log::error(e);
                return;
            }
        }
    };
}

pub fn start_interactive_dialogue() {
    set_up_terminal();
    log::header("Welcome to Renom");
    loop {
        match ok_or_quit!(request_workflow_selection_from_user()) {
            Workflow::RenameProject => ok_or_quit!(start_rename_project_workflow()),
            Workflow::RenameModule => ok_or_quit!(start_rename_module_workflow()),
        };
        if !user_wants_to_start_new_workflow() {
            break;
        }
    }
    log::basic("Thanks for using Renom.");
}

fn set_up_terminal() {
    log::check_support_for_colors();
}

fn request_workflow_selection_from_user() -> Result<Workflow, String> {
    let options = vec![Workflow::RenameProject, Workflow::RenameModule];
    Select::new("Choose a workflow:", options)
        .prompt()
        .map_err(|e| e.to_string())
}

fn user_wants_to_start_new_workflow() -> bool {
    Confirm::new("Would you like to start a new workflow?")
        .prompt()
        .unwrap_or(false)
}
