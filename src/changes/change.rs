use std::fmt::Display;

use super::{rename_file::RenameFile, AppendIniEntry, ReplaceInFile, SetIniEntry};

#[derive(Debug, PartialEq)]
pub enum Change {
    RenameFile(RenameFile),
    ReplaceInFile(ReplaceInFile),
    SetIniEntry(SetIniEntry),
    AppendIniEntry(AppendIniEntry),
}

impl Display for Change {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &*self {
            Change::RenameFile(p) => write!(f, "{}", &p),
            Change::ReplaceInFile(p) => write!(f, "{}", &p),
            Change::SetIniEntry(p) => write!(f, "{}", &p),
            Change::AppendIniEntry(p) => write!(f, "{}", &p),
        }
    }
}
