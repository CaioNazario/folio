//! One transaction around many index writes made through separate calls.

use crate::Result;
use crate::workspace::Workspace;

/// An open transaction on the workspace database. Dropping it without
/// [`Batch::commit`] rolls back, so an error halfway writes nothing.
pub struct Batch<'a> {
    workspace: &'a Workspace,
    done: bool,
}

impl Workspace {
    /// Starts a transaction that the following index writes join.
    pub(crate) fn begin_batch(&self) -> Result<Batch<'_>> {
        self.conn()
            .execute_batch("BEGIN")
            .map_err(|e| self.db_error(e))?;
        Ok(Batch {
            workspace: self,
            done: false,
        })
    }
}

impl Batch<'_> {
    pub fn commit(mut self) -> Result<()> {
        self.done = true;
        let workspace = self.workspace;
        workspace
            .conn()
            .execute_batch("COMMIT")
            .map_err(|e| workspace.db_error(e))
    }
}

impl Drop for Batch<'_> {
    fn drop(&mut self) {
        if !self.done {
            let _ = self.workspace.conn().execute_batch("ROLLBACK");
        }
    }
}
