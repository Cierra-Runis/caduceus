use std::collections::HashSet;

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
    #[serde(with = "time_0_3_offsetdatetime_as_bson_datetime")]
    pub updated_at: OffsetDateTime,
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
    ///
    /// The key is an opaque `String` rather than a backend-specific type on
    /// purpose: a GridFS `ObjectId`, an S3/MinIO object path, and a Git blob SHA
    /// are all just strings. Committing to `String` now (before any binary is
    /// persisted) means swapping the storage backend — or backing files onto a
    /// linked Git repository — never requires migrating stored documents.
    Binary { storage_key: String },
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

/// Canonicalize a client-supplied virtual-FS path, or `None` if it is not a
/// usable path within a project.
///
/// `path` is the primary key of a [`ProjectFile`] and the name the compiler
/// resolves (`#import`/`#image`), so it must be normalized before storage:
/// otherwise `logo.png`, `/logo.png`, and `./logo.png` are distinct rows that
/// alias to the same file in the compiler VFS. This trims and collapses
/// separators, drops `.` segments, and rejects anything that could escape the
/// project root (`..`), alias via backslashes, or smuggle control characters.
/// The result never has a leading/trailing slash (e.g. `chapters/intro.typ`).
pub fn normalize_vfs_path(raw: &str) -> Option<String> {
    if raw.contains('\\') || raw.chars().any(char::is_control) {
        return None;
    }
    let mut segments = Vec::new();
    for segment in raw.split('/') {
        match segment.trim() {
            "" | "." => continue,
            ".." => return None,
            segment => segments.push(segment),
        }
    }
    if segments.is_empty() {
        return None;
    }
    Some(segments.join("/"))
}

/// Pick a path that does not collide with `existing`, VS Code / browser style:
/// `logo.png` → `logo (1).png` → `logo (2).png`. Only the basename is suffixed;
/// any folder prefix and the file extension are preserved. Returns `path`
/// unchanged when it is already free. `path` is assumed already normalized (see
/// [`normalize_vfs_path`]).
pub fn dedupe_vfs_path(path: &str, existing: &HashSet<String>) -> String {
    if !existing.contains(path) {
        return path.to_string();
    }

    // Keep any folder prefix (with its trailing slash) untouched.
    let (dir, base) = match path.rfind('/') {
        Some(slash) => (&path[..=slash], &path[slash + 1..]),
        None => ("", path),
    };
    // Split off a real extension: a dot that is not the leading char of the
    // basename (so `.gitignore` stays whole, `archive.tar.gz` keeps `.gz`).
    let (stem, ext) = match base.rfind('.') {
        Some(dot) if dot > 0 => (&base[..dot], &base[dot..]),
        _ => (base, ""),
    };

    (1..)
        .map(|n| format!("{dir}{stem} ({n}){ext}"))
        .find(|candidate| !existing.contains(candidate))
        // `1..` is unbounded, so `find` always yields a free candidate.
        .expect("an unused suffix always exists")
}

#[derive(Serialize)]
pub struct ProjectPayload {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub owner_type: OwnerType,
    pub creator_id: String,
    pub files: Vec<ProjectFilePayload>,
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
            FileContent::Binary { storage_key } => FileContentPayload::Binary { storage_key },
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
            creator_id: project.creator_id.to_hex(),
            created_at: project.created_at,
            updated_at: project.updated_at,
            entry: project.entry.map(|id| id.to_hex()),
            pinned_version: project.pinned_version,
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_vfs_path_canonicalizes() {
        assert_eq!(normalize_vfs_path("logo.png").as_deref(), Some("logo.png"));
        assert_eq!(normalize_vfs_path("/logo.png").as_deref(), Some("logo.png"));
        assert_eq!(normalize_vfs_path("./logo.png").as_deref(), Some("logo.png"));
        assert_eq!(
            normalize_vfs_path("images//figure.svg").as_deref(),
            Some("images/figure.svg")
        );
        assert_eq!(
            normalize_vfs_path("  chapters/intro.typ  ").as_deref(),
            Some("chapters/intro.typ")
        );
        assert_eq!(
            normalize_vfs_path("a/b/../c.typ").as_deref(),
            None,
            "parent traversal is rejected outright"
        );
    }

    #[test]
    fn test_normalize_vfs_path_rejects_unusable() {
        assert_eq!(normalize_vfs_path(""), None);
        assert_eq!(normalize_vfs_path("/"), None);
        assert_eq!(normalize_vfs_path("   "), None);
        assert_eq!(normalize_vfs_path(".."), None);
        assert_eq!(normalize_vfs_path("a\\b.png"), None);
        assert_eq!(normalize_vfs_path("bad\u{0}name"), None);
    }

    #[test]
    fn test_dedupe_vfs_path_suffixes_basename() {
        let mut existing = HashSet::new();
        existing.insert("logo.png".to_string());

        // First collision gets " (1)" before the extension.
        assert_eq!(dedupe_vfs_path("logo.png", &existing), "logo (1).png");

        // Walks up until a free slot.
        existing.insert("logo (1).png".to_string());
        assert_eq!(dedupe_vfs_path("logo.png", &existing), "logo (2).png");

        // A free name is returned unchanged.
        assert_eq!(dedupe_vfs_path("fresh.png", &existing), "fresh.png");
    }

    #[test]
    fn test_dedupe_vfs_path_preserves_dir_and_extension() {
        let mut existing = HashSet::new();
        existing.insert("images/logo.png".to_string());
        assert_eq!(
            dedupe_vfs_path("images/logo.png", &existing),
            "images/logo (1).png"
        );

        // Multi-dot names keep only the final extension segment.
        existing.insert("archive.tar.gz".to_string());
        assert_eq!(
            dedupe_vfs_path("archive.tar.gz", &existing),
            "archive.tar (1).gz"
        );

        // Extensionless and dotfile names suffix at the end.
        existing.insert("Makefile".to_string());
        assert_eq!(dedupe_vfs_path("Makefile", &existing), "Makefile (1)");
        existing.insert(".gitignore".to_string());
        assert_eq!(dedupe_vfs_path(".gitignore", &existing), ".gitignore (1)");
    }
}
