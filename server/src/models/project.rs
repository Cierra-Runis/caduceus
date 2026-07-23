use bson::oid::ObjectId;
use bson::serde_helpers::time_0_3_offsetdatetime_as_bson_datetime;
use derive_more::Display;
use semver::Version;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use time::serde::rfc3339;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Display)]
pub enum OwnerType {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "team")]
    Team,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub owner_id: ObjectId,
    pub owner_type: OwnerType,
    pub creator_id: ObjectId,
    pub files: Vec<ProjectFile>,
    /// Explicitly-created directories, each a normalized path with no trailing
    /// slash (e.g. `chapters`, `assets/images`). Directories are otherwise
    /// *implied* by the files inside them; this list is what lets an **empty**
    /// folder exist and survive a reload, exactly like a real filesystem.
    /// Defaulted so documents written before this field existed still load.
    #[serde(default)]
    pub directories: Vec<String>,
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
}

/// A single node in the project's virtual file system.
///
/// The compiler never sees "a string" — it sees a file tree, because Typst
/// source resolves `#import "chapter.typ"`, `#image("logo.png")`, etc. against
/// the set of files. Therefore the key is [`ProjectFile::path`], not a flat
/// name.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectFile {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    /// Virtual-FS path used by the compiler to resolve imports/assets, e.g.
    /// `main.typ` or `chapters/intro.typ`. This is the file's primary key
    /// within the project, not a display label.
    pub path: String,
    /// The file's bytes — either inline UTF-8 source, or a pointer to a binary
    /// asset stored outside the document. See [`FileContent`].
    pub content: FileContent,
    /// Size in bytes. Primarily meaningful for binary assets; for inline text
    /// it is derivable from the content and kept for cheap listing/quotas.
    pub size: i64,
    pub version: i32,
    /// Present when this file is a usable font, carrying the family names it
    /// provides (see [`crate::font`]). Detected from the bytes at upload time
    /// so the client can register it into the Typst font book by family;
    /// `None` for every non-font file. Defaulted for documents written before
    /// this field existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<FontInfo>,
    #[serde(with = "time_0_3_offsetdatetime_as_bson_datetime")]
    pub updated_at: OffsetDateTime,
}

/// Font metadata attached to a font file: the family names it provides, keyed
/// on at font selection (`#set text(font: "…")`).
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FontInfo {
    pub families: Vec<String>,
}

const DEFAULT_MAIN_TYP: &str = "= Untitled\n\nStart writing Typst here.\n";
impl Default for ProjectFile {
    fn default() -> Self {
        Self {
            id: ObjectId::new(),
            path: "main.typ".to_string(),
            content: FileContent::Text {
                text: DEFAULT_MAIN_TYP.to_string(),
            },
            size: DEFAULT_MAIN_TYP.len() as i64,
            version: 0,
            font: None,
            updated_at: OffsetDateTime::now_utc(),
        }
    }
}

/// The content of a [`ProjectFile`]. Text and binary are split at the type
/// level so that M3 (image/asset upload) does not have to reshape the core
/// model: text lives inline in the Mongo document, binaries live elsewhere and
/// are referenced by key.
///
/// Reserved for M5 (real-time collaboration): a `Crdt { state: Bson binary }`
/// variant holding a Yjs/yrs snapshot. The tagged shape leaves room for it
/// without breaking stored documents.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum FileContent {
    /// UTF-8 source stored inline in the document (`.typ`, `.bib`, `.csl`, …).
    Text { text: String },
    /// A binary asset (image, font, …) stored outside the document — in GridFS
    /// or object storage — and referenced here by its storage key.
    Binary { storage_key: ObjectId },
}

impl FileContent {
    /// Discriminator exposed to the API so clients can pick an icon / decide
    /// whether to fetch text without downloading the whole payload.
    pub fn kind(&self) -> FileKind {
        match self {
            FileContent::Text { .. } => FileKind::Text,
            FileContent::Binary { .. } => FileKind::Binary,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum FileKind {
    Text,
    Binary,
}

#[derive(Serialize)]
pub struct ProjectPayload {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub owner_type: OwnerType,
    pub creator_id: String,
    pub files: Vec<ProjectFilePayload>,
    pub directories: Vec<String>,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
    pub entry: Option<String>,
    pub pinned_version: Option<Version>,
}

/// File metadata for listings. Deliberately does NOT inline `content`: the
/// directory/tree view only needs path + kind, and the body is fetched
/// per-file when a tab is opened.
#[derive(Serialize)]
pub struct ProjectFilePayload {
    pub id: String,
    pub path: String,
    pub kind: FileKind,
    pub size: i64,
    pub version: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<FontInfo>,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl From<Project> for ProjectPayload {
    fn from(project: Project) -> Self {
        ProjectPayload {
            id: project.id.to_hex(),
            name: project.name,
            owner_id: project.owner_id.to_hex(),
            owner_type: project.owner_type,
            files: project
                .files
                .into_iter()
                .map(ProjectFilePayload::from)
                .collect(),
            directories: project.directories,
            creator_id: project.creator_id.to_hex(),
            created_at: project.created_at,
            updated_at: project.updated_at,
            entry: project.entry.map(|id| id.to_hex()),
            pinned_version: project.pinned_version,
        }
    }
}

impl From<ProjectFile> for ProjectFilePayload {
    fn from(file: ProjectFile) -> Self {
        ProjectFilePayload {
            id: file.id.to_hex(),
            path: file.path,
            kind: file.content.kind(),
            size: file.size,
            version: file.version,
            font: file.font,
            updated_at: file.updated_at,
        }
    }
}

/// Editor-facing payload for opening a single project. Unlike [`ProjectPayload`]
/// (used by the list endpoints), this inlines text file content: the Typst
/// compiler resolves `#import`/`#image` across the *entire* file tree, not just
/// the focused tab, so the client needs the whole virtual FS up front. Lazy
/// per-file loading would not serve the preview.
#[derive(Serialize)]
pub struct ProjectDetailPayload {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub owner_type: OwnerType,
    pub creator_id: String,
    pub files: Vec<ProjectFileDetailPayload>,
    pub directories: Vec<String>,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
    /// The compile entry, as the file's id (hex). The client resolves it to a
    /// path against `files` — id is the stable key, path can be renamed.
    pub entry: Option<String>,
    pub pinned_version: Option<Version>,
}

/// A single file with its content inlined, for the editor's initial load.
#[derive(Serialize)]
pub struct ProjectFileDetailPayload {
    pub id: String,
    pub path: String,
    pub content: FileContentPayload,
    pub size: i64,
    pub version: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<FontInfo>,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// Wire form of [`FileContent`]. Text is inlined so the compiler can use it
/// immediately; a binary stays a reference (`storageKey`) — its bytes are
/// served separately once asset delivery lands (M3).
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum FileContentPayload {
    Text {
        text: String,
    },
    Binary {
        #[serde(rename = "storageKey")]
        storage_key: String,
    },
}

impl From<FileContent> for FileContentPayload {
    fn from(content: FileContent) -> Self {
        match content {
            FileContent::Text { text } => FileContentPayload::Text { text },
            FileContent::Binary { storage_key } => FileContentPayload::Binary {
                storage_key: storage_key.to_hex(),
            },
        }
    }
}

impl From<ProjectFile> for ProjectFileDetailPayload {
    fn from(file: ProjectFile) -> Self {
        ProjectFileDetailPayload {
            id: file.id.to_hex(),
            path: file.path,
            content: file.content.into(),
            size: file.size,
            version: file.version,
            font: file.font,
            updated_at: file.updated_at,
        }
    }
}

/// Returned after a file content save. Just the freshly bumped version and
/// timestamp — enough for the client to track save state / optimistic
/// concurrency without echoing the text it just sent.
#[derive(Serialize)]
pub struct UpdateFilePayload {
    pub id: String,
    pub version: i32,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl From<ProjectFile> for UpdateFilePayload {
    fn from(file: ProjectFile) -> Self {
        UpdateFilePayload {
            id: file.id.to_hex(),
            version: file.version,
            updated_at: file.updated_at,
        }
    }
}

impl From<Project> for ProjectDetailPayload {
    fn from(project: Project) -> Self {
        ProjectDetailPayload {
            id: project.id.to_hex(),
            name: project.name,
            owner_id: project.owner_id.to_hex(),
            owner_type: project.owner_type,
            files: project
                .files
                .into_iter()
                .map(ProjectFileDetailPayload::from)
                .collect(),
            directories: project.directories,
            creator_id: project.creator_id.to_hex(),
            created_at: project.created_at,
            updated_at: project.updated_at,
            entry: project.entry.map(|id| id.to_hex()),
            pinned_version: project.pinned_version,
        }
    }
}
