//! What the sidebar footer shows about the open workspace (Design spec 03,
//! "Rodapé").

use serde::Serialize;
use ts_rs::TS;

use crate::documents::WorkspaceTotals;

/// The workspace as the footer describes it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WorkspaceSummary {
    /// Where it is, as the user reads it ("~/Documentos/Folio").
    pub location: String,
    pub totals: WorkspaceTotals,
    /// The system user's name, for the avatar's initial.
    pub user_name: String,
}
