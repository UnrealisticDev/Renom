use std::path::Path;

use crate::{
    changes::{Change, Revert},
    presentation::log,
};

pub struct Engine {
    history: Vec<(Change, Revert)>,
}

impl Engine {
    pub fn new() -> Self {
        Self { history: vec![] }
    }

    /// Execute a series of changes in sequential order and stores the
    /// applied changes in history with appropriate revert actions.
    /// Upon error, it will halt execution and return the error.
    pub fn execute(
        &mut self,
        changeset: Vec<Change>,
        backup_dir: impl AsRef<Path>,
    ) -> Result<(), String> {
        for change in changeset {
            log::basic(format!("Apply: {}", change));
            self.execute_single(change, backup_dir.as_ref())?;
        }
        Ok(())
    }

    fn execute_single(&mut self, change: Change, backup_dir: &Path) -> Result<(), String> {
        match change.apply(backup_dir) {
            Ok(revert) => {
                self.history.push((change, revert));
                Ok(())
            }
            Err(err) => Err(err.to_string()),
        }
    }

    /// Revert entire history of actions.
    /// Upon error, it will halt execution and return the error.
    pub fn revert(&mut self) -> Result<(), String> {
        while let Some((change, revert)) = self.history.pop() {
            log::basic(format!("Revert: {}", change));
            revert().map_err(|err| err.to_string())?;
        }
        Ok(())
    }
}
