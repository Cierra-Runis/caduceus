import { $typst } from '@myriaddreamin/typst.ts';

// Client-side Typst compiler (reflexo WASM). This is the single place that
// initialises the compiler/renderer and turns the project's files into preview
// SVG, shared by every consumer (the project preview panel today; LSP/export
// later).
//
// Wasm + default fonts are pulled from the CDN, version-pinned to the installed
// JS API so the glue and the wasm never skew. The real project fonts are
// user-uploaded assets fed in separately (M3); this default set only covers the
// Latin core.
const TYPST_VERSION = '0.7.0';

let initialized = false;

/// A non-source file the document may load as *bytes* — an image for
/// `#image(...)`, a dataset for `#read(...)`/`#csv(...)`, a `.bib`, etc. These
/// are registered as shadow files so the sandboxed compiler can read them;
/// without this, any such load fails with "failed to load file (access
/// denied) / cannot read file outside of project root".
export interface TypstAssetFile {
  bytes: Uint8Array;
  /// Virtual-FS path, e.g. `assets/logo.png`.
  path: string;
}

export interface TypstSourceFile {
  /// Virtual-FS path, e.g. `main.typ` or `chapters/intro.typ`.
  path: string;
  text: string;
}

/// Compile a project to an SVG string, starting from `entryPath`.
///
/// The whole tree is fed to the compiler (not just the focused file) because
/// Typst resolves `#import`/`#image`/`#read` across every file: `.typ` sources
/// go in as editable source text, everything else as shadowed bytes. Paths are
/// normalised to an absolute VFS root so relative loads resolve the same way
/// regardless of which file is the entry.
export async function compileProject(
  entryPath: string,
  sources: TypstSourceFile[],
  assets: TypstAssetFile[] = [],
): Promise<string> {
  ensureInit();
  for (const file of sources) {
    await $typst.addSource(abs(file.path), file.text);
  }
  for (const asset of assets) {
    await $typst.mapShadow(abs(asset.path), asset.bytes);
  }
  return $typst.svg({ mainFilePath: abs(entryPath) });
}

/// Compile a project's text files to PDF bytes, starting from `entryPath`.
///
/// Mirrors `compileProject`, swapping the renderer's `.svg()` for the
/// compiler's `.pdf()`. `$typst.pdf()` types its result as possibly
/// `undefined` (no successful document), which we surface as a thrown error
/// so callers only ever deal with bytes or a caught failure.
export async function compileProjectToPdf(
  entryPath: string,
  sources: TypstSourceFile[],
  assets: TypstAssetFile[] = [],
): Promise<Uint8Array> {
  ensureInit();
  for (const file of sources) {
    await $typst.addSource(abs(file.path), file.text);
  }
  for (const asset of assets) {
    await $typst.mapShadow(abs(asset.path), asset.bytes);
  }
  const pdf = await $typst.pdf({ mainFilePath: abs(entryPath) });
  if (!pdf) throw new Error('Typst compiler produced no PDF output.');
  return pdf;
}

function abs(path: string): string {
  return `/${path.replace(/^\/+/, '')}`;
}

function ensureInit() {
  if (initialized) return;
  initialized = true;
  $typst.setCompilerInitOptions({
    getModule: () =>
      `https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-web-compiler@${TYPST_VERSION}/pkg/typst_ts_web_compiler_bg.wasm`,
  });
  $typst.setRendererInitOptions({
    getModule: () =>
      `https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-renderer@${TYPST_VERSION}/pkg/typst_ts_renderer_bg.wasm`,
  });
}
