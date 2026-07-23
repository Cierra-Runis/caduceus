# File Storage & the Project File Tree

How a project's files are modelled, stored, mutated, and rendered — and where a
future GitHub-sync layer plugs in.

## Model

A project owns a **virtual file system**, not a flat blob of text:

- `Project.files: Vec<ProjectFile>` — every file, keyed by `path`
  (e.g. `main.typ`, `chapters/intro.typ`). The Typst compiler resolves
  `#import` / `#image` across this whole set, so `path` — not a display name —
  is the primary key.
- `Project.directories: Vec<String>` — **explicitly-created** directories. A
  directory is otherwise *implied* by the files inside it; this list is what
  lets an **empty** folder exist and survive a reload, exactly like a real
  filesystem. Deleting a folder prunes both the files under it and the matching
  entries here.
- `ProjectFile.content` is `Text { text }` (UTF-8 inlined in the Mongo
  document) or `Binary { storage_key }` (bytes live in object storage, only the
  key is stored here).
- `Project.entry` — the file the compiler starts from, addressed by **id** so a
  rename never breaks it. Deleting the entry (or a folder containing it) clears
  it.

### Path rules (`server/src/models/path.rs`)

The single authority for a legal path, enforced server-side on every create /
rename / upload and mirrored client-side (`app/src/lib/path.ts`) for instant
feedback:

- relative to the project root, `/`-separated, within length/depth caps;
- no empty segment (rejects a leading, trailing, or doubled `/`);
- no `.` or `..` segment (no traversal);
- no segment with a backslash, control character, or leading/trailing
  whitespace.

**Uniqueness** is enforced in the service layer: a path may not collide with an
existing file or directory, and a name may not be both a file and a directory
(`a` and `a/b.typ` cannot coexist). This is what "no duplicate names" means
here.

## HTTP API

All under `/api/project/{id}` behind the JWT middleware; access is checked
against project ownership/membership on every call.

| Method   | Path                       | Purpose                                  |
| -------- | -------------------------- | ---------------------------------------- |
| `POST`   | `/file`                    | create an empty text file (`{ path }`)   |
| `PATCH`  | `/file/{file_id}`          | rename / move a file (`{ path }`)        |
| `DELETE` | `/file/{file_id}`          | delete a file (frees its blob if binary) |
| `GET`    | `/file/{file_id}/raw`      | stream a binary asset's bytes            |
| `PUT`    | `/file/{file_id}`          | save text content (whole-buffer)         |
| `POST`   | `/folder`                  | create an empty folder (`{ path }`)      |
| `DELETE` | `/folder`                  | delete a folder + subtree (`{ path }`)   |
| `POST`   | `/upload`                  | multipart binary upload (proxied)        |
| `POST`   | `/entry`                   | set the compile entry (`{ file_id }`)    |

Errors map to `400` (invalid path), `404` (missing file/project), `409`
(path conflict), `403` (access denied).

## Object storage (MinIO / S3)

Binary bytes never sit in the Mongo document — a project document is loaded
whole into the editor, and an image would bloat every load. Instead:

- `storage::ObjectStore` is a small trait (`put` / `get` / `delete`).
  Production wires `MinioObjectStore` (S3-compatible, path-style); tests use
  `InMemoryObjectStore`. Nothing above the storage module knows which backend
  is in play.
- **Key layout:** `projects/{project_id}/{storage_key}`. Namespacing by project
  makes a per-project export (or GitHub sync, below) a simple prefix listing.
- **Uploads are proxied** through the server (chosen over presigned
  direct-to-MinIO): the server authenticates, validates *every* target path
  against the tree **before** a byte is written, then streams to MinIO. A
  rejected batch therefore never orphans a blob, and MinIO needs no bucket-CORS
  setup. The multipart body names each part after its target path
  (`form.append(targetPath, file)`); the server reads the field name.
- **Text vs. binary is decided by content, not extension.** An uploaded file
  whose bytes are valid UTF-8 with no NUL byte (and under a 1 MiB inline cap) is
  stored as inline `Text` — so an uploaded `README.md` or `.typ` is a normal
  editable, collaborative file, not a dead binary. Everything else (images,
  fonts, oversized text) becomes a `Binary` in object storage.

Run MinIO locally with `docker-compose.yml`; defaults match
`StorageConfig` so a fresh checkout works unconfigured.

## Frontend

`FileExplorerPanel` renders a VS Code-style tree (titled **文件 / Files**):
folders expand/collapse, files show type icons and an `entry` badge, right-click
opens a context menu (new file/folder, rename, delete, set-as-entry, download).
The tree is server-authoritative — the panel holds the project detail in SWR and
revalidates after every change rather than guessing locally.

`UploadDialog` stages files entirely client-side (drag-drop, file picker, or
folder picker via `webkitdirectory`): each staged row shows the path it will
take relative to the project root and any conflict, and **nothing is uploaded
until Upload is pressed**.

## Future: GitHub sync

The model is deliberately git-shaped, so syncing a project to/from a Git repo is
additive, not a rewrite:

- `path` maps 1:1 to a git tree path; the path rules already reject anything git
  couldn't represent.
- Text lives inline (→ git blob directly); binaries are content in object
  storage keyed per project (→ mirror to git blobs / Git LFS without touching
  the rest of the app).
- `directories` preserves empty folders; git doesn't track those, so a sync
  layer would materialize them with a `.gitkeep` on export and read them back on
  import.

The intended seam is a `sync` service that walks a project's `files` +
`directories`, reads text inline and binaries via `ObjectStore::get`, and writes
a tree/commit — with the reverse for import. No schema change is required to add
it.

## Known limitations / follow-ups

- **Folder rename/move** is not yet implemented (it needs an atomic multi-file
  path rewrite); the UI surfaces this rather than silently no-oping. Files can
  be renamed/moved individually.
- **Newly-created text files** created mid-session are not picked up by an
  already-live collaboration room's persistence map until the room re-seeds
  (next open). The file exists and is editable immediately; its edits persist
  after a reload. Wiring dynamic file registration into the live room is a
  separate change to `handler/ws.rs`.
- Binary and data assets are fed to the client-side Typst compiler as shadow
  files (`ObjectStore` bytes for binaries, inline text encoded for data files),
  so `#image(...)` / `#read(...)` resolve in both the live preview and PDF
  export.
- **Fonts** are detected server-side at upload time by their binary sfnt
  signature — not their extension — (`.ttf`/`.otf`/`.ttc`/`.otc`; WOFF/WOFF2 are
  excluded as Typst can't use them), and their family names are parsed from the
  font's `name` table and stored on the `ProjectFile`. The client registers a
  font's bytes into the compiler's font book by family (not as a path-addressed
  shadow file, since Typst selects fonts by family name), so
  `#set text(font: "…")` resolves against uploaded fonts. Fonts and the font
  family names are shown in the file tree.
