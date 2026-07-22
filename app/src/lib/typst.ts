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

export interface TypstBinaryFile {
  /// Raw bytes of the asset, mapped into the compiler VFS so `#image(...)` and
  /// custom fonts resolve.
  bytes: Uint8Array;
  /// Virtual-FS path, e.g. `logo.png` or `images/figure.svg`.
  path: string;
}

export interface TypstSourceFile {
  /// Virtual-FS path, e.g. `main.typ` or `chapters/intro.typ`.
  path: string;
  text: string;
}

/// Compile a project to an SVG string, starting from `entryPath`.
///
/// The whole file tree is fed to the compiler (not just the focused file)
/// because Typst resolves `#import`/`#image` across every file.
export async function compileProject(
  entryPath: string,
  files: TypstSourceFile[],
  assets: TypstBinaryFile[] = [],
): Promise<string> {
  ensureInit();
  await loadVfs(files, assets);
  return $typst.svg({ mainFilePath: abs(entryPath) });
}

/// Compile a project to PDF bytes, starting from `entryPath`.
///
/// Mirrors `compileProject`, swapping the renderer's `.svg()` for the
/// compiler's `.pdf()`. `$typst.pdf()` types its result as possibly
/// `undefined` (no successful document), which we surface as a thrown error
/// so callers only ever deal with bytes or a caught failure.
export async function compileProjectToPdf(
  entryPath: string,
  files: TypstSourceFile[],
  assets: TypstBinaryFile[] = [],
): Promise<Uint8Array> {
  ensureInit();
  await loadVfs(files, assets);
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

/// Seed the compiler's virtual file system from the project's whole file tree.
///
/// The shadow map is reset first so each compile is hermetic: a renamed or
/// deleted file never lingers from a previous run. Text goes in as sources,
/// binaries (`assets`) as raw shadow files. Paths are normalised to an absolute
/// VFS root so relative `#import`/`#image` resolve the same way regardless of
/// which file is the entry.
async function loadVfs(
  files: TypstSourceFile[],
  assets: TypstBinaryFile[],
): Promise<void> {
  await $typst.resetShadow();
  for (const file of files) {
    await $typst.addSource(abs(file.path), file.text);
  }
  for (const asset of assets) {
    await $typst.mapShadow(abs(asset.path), asset.bytes);
  }
}
