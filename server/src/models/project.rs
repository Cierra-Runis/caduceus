use std::collections::HashMap;

use bson::oid::ObjectId;
use bson::serde_helpers::time_0_3_offsetdatetime_as_bson_datetime;
use derive_more::Display;
use semver::Version;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use time::serde::rfc3339;

use crate::models::tree::{NodeId, ProjectionEntry};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Display)]
pub enum OwnerType {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "team")]
    Team,
}

/// When the editor materializes a file's live text into a durable
/// content-addressed blob — mirroring VS Code's `files.autoSave`. Note this
/// governs *blob materialization*, not durability: every keystroke is already
/// streamed to the server over the CRDT and snapshotted, so `Off` never risks
/// losing synced text — it only defers minting a blob until an explicit save.
/// The trigger itself is detected on the client (only it knows about editor /
/// window focus and keystroke timing); the server just flushes on request.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum AutoSavePolicy {
    /// Never flush automatically; the user saves by hand (e.g. Ctrl/Cmd+S).
    Off,
    /// Flush a short debounce after the last edit (see `auto_save_delay`).
    AfterDelay,
    /// Flush when focus leaves the edited file (switching tabs, blurring).
    #[default]
    OnFocusChange,
    /// Flush when the browser window / tab loses focus.
    OnWindowChange,
}

/// Project-level editor settings, shared by every collaborator. Every field is
/// `#[serde(default)]` so a project document written before this existed still
/// deserializes (missing settings become the defaults).
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettings {
    #[serde(default)]
    pub auto_save: AutoSavePolicy,
    /// Debounce in milliseconds for `AutoSavePolicy::AfterDelay` (VS Code's
    /// `files.autoSaveDelay`). Ignored by the other policies.
    #[serde(default = "default_auto_save_delay")]
    pub auto_save_delay: u32,
}

fn default_auto_save_delay() -> u32 {
    1000
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            auto_save: AutoSavePolicy::default(),
            auto_save_delay: default_auto_save_delay(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub owner_id: ObjectId,
    pub owner_type: OwnerType,
    pub creator_id: ObjectId,
    #[serde(with = "time_0_3_offsetdatetime_as_bson_datetime")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time_0_3_offsetdatetime_as_bson_datetime")]
    pub updated_at: OffsetDateTime,
    /// The file the Typst compiler starts from (the project's "main" file).
    /// Project-level and shared by every collaborator — this is the single
    /// source of truth for "what gets compiled" (server-side PDF export, etc.).
    /// NOT to be confused with the per-user *focus* file (which tab someone is
    /// currently looking at); focus is session-level and lives on the client /
    /// awareness channel, never in this document.
    pub entry: Option<ObjectId>,
    pub pinned_version: Option<Version>,
    /// Editor settings shared by every collaborator (e.g. the auto-save
    /// policy). Defaulted when absent from an older stored document.
    #[serde(default)]
    pub settings: ProjectSettings,
    /// The id-keyed projection of the CRDT file tree — a rebuildable cache the
    /// collaboration room refreshes on persist (see `ProjectTree::projection`).
    /// The authoritative structure is the Y.Doc snapshot; this mirrors it for
    /// cheap metadata reads. Defaulted (empty) when absent from an older
    /// document, and rebuilt on the next persist.
    #[serde(default)]
    pub tree: HashMap<NodeId, ProjectionEntry>,
}

/// The Typst source seeded into a new project's entry file (`main.typ`).
pub const DEFAULT_MAIN_TYP: &str = "= Untitled\n\nStart writing Typst here.\n";

#[derive(Serialize)]
pub struct ProjectPayload {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub owner_type: OwnerType,
    pub creator_id: String,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
    pub entry: Option<String>,
    pub pinned_version: Option<Version>,
}

impl From<Project> for ProjectPayload {
    fn from(project: Project) -> Self {
        ProjectPayload {
            id: project.id.to_hex(),
            name: project.name,
            owner_id: project.owner_id.to_hex(),
            owner_type: project.owner_type,
            creator_id: project.creator_id.to_hex(),
            created_at: project.created_at,
            updated_at: project.updated_at,
            entry: project.entry.map(|id| id.to_hex()),
            pinned_version: project.pinned_version,
        }
    }
}

/// Editor-facing payload for opening a single project. Carries the file **tree**
/// (structure + blob refs), id-keyed — it does **not** inline text: the editor
/// reads text from the CRDT, and other consumers (e.g. project download) fetch
/// the referenced blobs on demand.
#[derive(Serialize)]
pub struct ProjectDetailPayload {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub owner_type: OwnerType,
    pub creator_id: String,
    pub tree: HashMap<NodeId, ProjectionEntry>,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
    /// The compile entry, as the file's id (hex). The client resolves it to a
    /// path against `tree` — id is the stable key, path can be renamed.
    pub entry: Option<String>,
    pub pinned_version: Option<Version>,
    /// Project-level editor settings (auto-save policy, …).
    pub settings: ProjectSettings,
}

impl From<Project> for ProjectDetailPayload {
    fn from(project: Project) -> Self {
        ProjectDetailPayload {
            id: project.id.to_hex(),
            name: project.name,
            owner_id: project.owner_id.to_hex(),
            owner_type: project.owner_type,
            tree: project.tree,
            creator_id: project.creator_id.to_hex(),
            created_at: project.created_at,
            updated_at: project.updated_at,
            entry: project.entry.map(|id| id.to_hex()),
            pinned_version: project.pinned_version,
            settings: project.settings,
        }
    }
}
