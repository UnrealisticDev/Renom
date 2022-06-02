use super::{rename_file::RenameFile, AppendIniEntry, ReplaceInFile, SetIniEntry};

#[derive(Debug, PartialEq)]
pub enum Change {
    RenameFile(RenameFile),
    ReplaceInFile(ReplaceInFile),
    SetIniEntry(SetIniEntry),
    AppendIniEntry(AppendIniEntry),
}
