use std::fmt::Display;

pub enum Workflow {
    RenameProject,
    RenameModule,
}

impl Display for Workflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &*self {
            Workflow::RenameProject => write!(f, "Rename a project"),
            Workflow::RenameModule => write!(f, "Rename a module"),
        }
    }
}
