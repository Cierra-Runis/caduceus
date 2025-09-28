use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum Message {
    Edit(ModelContentChangedEvent),
    CursorSelectionChanged(CursorSelectionChangedEvent),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelContentChangedEvent {
    pub changes: Vec<ModelContentChange>,
    pub eol: String,
    pub version_id: u32,
    pub is_undoing: bool,
    pub is_redoing: bool,
    pub is_flush: bool,
    pub is_eol_change: bool,
    pub detailed_reasons_change_lengths: Option<Vec<u32>>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelContentChange {
    pub range: Range,
    pub range_offset: u32,
    pub range_length: u32,
    pub text: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    pub start_line_number: u32,
    pub start_column: u32,
    pub end_line_number: u32,
    pub end_column: u32,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CursorSelectionChangedEvent {
    pub selection: Selection,
    pub secondary_selections: Vec<Selection>,
    pub model_version_id: u32,
    pub old_selections: Option<Vec<Selection>>,
    pub old_model_version_id: u32,
    pub source: String,
    pub reason: CursorChangeReason,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Selection {
    start_line_number: u32,
    start_column: u32,
    end_line_number: u32,
    end_column: u32,
    selection_start_line_number: u32,
    selection_start_column: u32,
    position_line_number: u32,
    position_column: u32,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum CursorChangeReason {
    NotSet = 0,
    ContentFlush = 1,
    RecoverFromMarkers = 2,
    Explicit = 3,
    Paste = 4,
    Undo = 5,
    Redo = 6,
}
