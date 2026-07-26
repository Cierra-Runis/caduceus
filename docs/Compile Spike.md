# Server-Side Compile Spike (Slice 0)

Feasibility check for moving Typst compilation/preview from client-side WASM to
a server-side `tinymist` subprocess (Route B). Goal was to rule out the
biggest unknowns — can we run the binary, do fonts resolve, how fast is a
compile — before writing any integration code. No server code changed in this
slice.

## Setup

- Platform tested: Windows (x86_64-pc-windows-msvc), the dev sandbox available
  for this spike. Production targets x86_64 Linux (glibc); the CLI surface
  exercised here (`compile`) is platform-agnostic, but font-resolution and
  timing numbers below are Windows-only and should be re-measured once a
  Linux binary is in CI.
- Binary: `tinymist` **v0.13.27** (`Typst Version: 0.14.0`), one of several
  pre-downloaded release builds. Acquisition path confirmed: GitHub releases
  ship raw platform binaries (`tinymist-<target>`), not a crates.io binary —
  `cargo install tinymist` fails with "no binaries" because that crate is
  library-only. See "Version pinning" below for the acquisition-automation
  implication.
- No `--font-path` / `--ignore-system-fonts` flags were passed; tinymist found
  system fonts on its own.

## What was verified

1. **The binary runs and has a stable-shaped CLI (in recent versions).**
   `tinymist compile <input> [output]` mirrors `typst-cli compile`: format is
   inferred from the output extension, or set explicitly with `-f`. `--root`
   sets the project root for absolute/relative path resolution; `--font-path`
   / `--ignore-system-fonts` control fonts; `TYPST_ROOT` and
   `TYPST_FONT_PATHS` env vars work as overrides.

2. **All three static export formats work out of the box**: a trivial
   `= Hello` document compiled cleanly to SVG, PDF, and PNG with system fonts,
   no font-related errors or fallback-glyph warnings.

3. **Multi-file projects resolve correctly.** A `main.typ` importing
   `helper.typ` via `#import "helper.typ": greet`, compiled with
   `--root <dir>`, resolved the import and rendered without issue. This is
   the mechanism Slice 1's workspace materialization depends on: write every
   node's text/blob to its `path_of`-derived path under a temp root, then
   point `--root` at that temp dir.

4. **Diagnostics are compiler-owned, as the architecture assumes.** A missing
   import and a syntax error both produced human-readable, file/line/column
   diagnostics on stderr and exited non-zero — nothing crashed, no partial
   output was written. Sample:

   ```
   error: file not found (searched at ...\err\missing.typ)
     ┌─ ...\err\broken.typ:1:8
     │
   1 │ #import "missing.typ": nope
     │         ^^^^^^^^^^^^^
   ```

   Open question for Slice 1: this `compile` subcommand has no
   `--diagnostic-format json` flag (checked `--help`; not present as of
   v0.13.27). We'll either parse this text format or drive tinymist through
   its LSP `textDocument/publishDiagnostics` notifications instead, which are
   structured JSON. Worth deciding explicitly before building the endpoint,
   not discovering it mid-implementation.

## Timing

One-shot process-per-compile, no daemon, five back-to-back invocations of the
same trivial document plus one multi-file/import case, all on the same
machine:

| Run | Document | Wall time |
| --- | --- | --- |
| 1 (cold) | `= Hello` + one paragraph | 103ms |
| 2 | same | 102ms |
| 3 | same | 101ms |
| 4 | same | 103ms |
| 5 | same | 99ms |
| cold | multi-file, `#import` + 2×`#lorem` | 107ms |
| warm ×3 | same | 106–113ms |

Takeaway: there is **no meaningful cold-vs-warm gap** for a fresh subprocess
per request — every invocation, including the very first, lands around
100ms. That's small enough that Slice 1 can start with the simplest possible
model (spawn `tinymist compile` per compile request, no persistent worker)
and defer daemon/session reuse to Slice 3 as planned, rather than treating it
as a prerequisite. These numbers are for a near-empty document on Windows;
larger real documents and Linux-in-CI numbers should be re-checked before
treating "~100ms" as a real SLA.

## Version pinning — a constraint this spike surfaces

The requirement that the project support freely switching tinymist/Typst
versions needs to be scoped, not open-ended. Checking the CLI shape across
the pre-downloaded releases:

| Version | `compile` subcommand |
| --- | --- |
| v0.11.0 | absent (options-only CLI, no subcommands at all) |
| v0.11.10 | present, described as "Run Compile Server" |
| v0.11.20 – v0.12.10 | **absent again** |
| v0.12.22 onward | present, described as `Runs compile command like typst-cli compile` (current shape) |

So `compile` is not a stable feature across tinymist's history — it
disappeared for a stretch between v0.11.20 and v0.12.10ish before coming back
in its current form. Implication for later slices: "free version switching"
should mean *switching among versions whose CLI we've verified against our
integration contract* (currently: `v0.12.22+`), not literally any tagged
release. A cheap runtime guard — run `tinymist --help` (or `probe`) once when
a version is registered/selected and check for the subcommands we depend on
(`compile`, `lsp`) — is worth building into whatever loads the configured
binary path, so an incompatible pin fails fast with a clear error instead of
a confusing runtime crash.

## Acquisition for CI / deployment

No binary should be fetched at runtime (a past sandbox environment had GitHub
egress blocked entirely, which is the failure mode to design around). Instead:

- Deployment images and CI need the pinned binary baked in at build time.
- `typst-cli` (crates.io, has real binaries) is a useful smoke-test/reference
  for the underlying typst core, independent of tinymist's own CLI stability,
  but is not a substitute for tinymist itself (no LSP, no preview protocol).
- A small fetch script (per-OS: `tinymist-x86_64-unknown-linux-gnu` /
  `tinymist-x86_64-pc-windows-msvc` asset naming on the release page) that
  downloads and caches a given version tag is the right shape for both "CI
  installs the pinned version" and "let the project pick a version" — this is
  Slice 1/CI work, not built in this spike.

## Not covered in this slice (by design)

- Binary asset materialization (`ProjectStore::get_blob` → workspace file) —
  deferred to Slice 1, which has the actual `ProjectTree` to materialize.
  This spike only proved path resolution (`--root` + relative `#import`)
  works, which is the mechanism that materialization relies on.
- Linux binary / fontconfig behavior — needs to be re-verified once a Linux
  tinymist binary is available in this environment or in CI.
- LSP mode and the preview (typst-preview) protocol — out of scope for
  Slice 0 per the slice plan; `compile`-only was sufficient to answer "can we
  run this and how fast."

## Conclusion

Route B is feasible from a build/runtime standpoint: the binary runs, fonts
resolve without extra configuration, multi-file path resolution works the way
Slice 1's materialization step needs it to, and per-compile latency for a
trivial document is small and flat (~100ms, cold or warm) — cheap enough that
Slice 1 doesn't need to solve process reuse to be usable. The main non-obvious
risk this spike found is CLI-shape drift across tinymist versions, which
narrows "freely switchable versions" to a verified-compatible range rather
than the full release history.
