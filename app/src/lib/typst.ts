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

export interface TypstSourceFile {
  /// Virtual-FS path, e.g. `main.typ` or `chapters/intro.typ`.
  path: string;
  text: string;
}

/// Compile a project's text files to an SVG string, starting from `entryPath`.
///
/// The whole text tree is fed to the compiler (not just the focused file)
/// because Typst resolves `#import`/`#image` across every file. Paths are
/// normalised to an absolute VFS root so relative imports resolve the same way
/// regardless of which file is the entry. Binary assets are not wired yet (M3).
export async function compileProject(
  entryPath: string,
  files: TypstSourceFile[],
): Promise<string> {
  ensureInit();
  for (const file of files) {
    await $typst.addSource(abs(file.path), file.text);
  }
  return $typst.svg({ mainFilePath: abs(entryPath) });
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
