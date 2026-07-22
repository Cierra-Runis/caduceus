import { $typst } from '@myriaddreamin/typst.ts';
import { TypstSnippet } from '@myriaddreamin/typst.ts/contrib/snippet';

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

// Font-file extensions we register with the compiler's font book (see
// `registerFonts`). Lower-cased before comparison so `LOGO.TTF` still matches.
const FONT_EXTENSIONS = ['.ttf', '.otf', '.ttc', '.otc'];

let initialized = false;

// Paths of font assets already handed to the compiler's font book. A font only
// needs registering once; the preview recompiles on every keystroke (200ms
// debounce) with the same asset set, and re-registering per keystroke would be
// wasteful — and, once the compiler is built, impossible (see `registerFonts`).
const registeredFontPaths = new Set<string>();

// Flips true once the compiler has actually been built. Font providers are
// baked into the compiler at init time and the global compiler instance is
// initialised exactly once, so fonts must be registered before this point.
let compilerBuilt = false;

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

function isFontAsset(path: string): boolean {
  const lower = path.toLowerCase();
  return FONT_EXTENSIONS.some((ext) => lower.endsWith(ext));
}

/// Seed the compiler's virtual file system from the project's whole file tree.
///
/// The shadow map is reset first so each compile is hermetic: a renamed or
/// deleted file never lingers from a previous run. Text goes in as sources,
/// binaries (`assets`) as raw shadow files. Paths are normalised to an absolute
/// VFS root so relative `#import`/`#image` resolve the same way regardless of
/// which file is the entry.
///
/// Fonts are registered with the compiler's font book *before* the first VFS
/// call, because `resetShadow` below is what lazily builds the compiler and
/// font providers only take effect at build time.
async function loadVfs(
  files: TypstSourceFile[],
  assets: TypstBinaryFile[],
): Promise<void> {
  registerFonts(assets);
  await $typst.resetShadow();
  compilerBuilt = true;
  for (const file of files) {
    await $typst.addSource(abs(file.path), file.text);
  }
  for (const asset of assets) {
    await $typst.mapShadow(abs(asset.path), asset.bytes);
  }
}

/// Register uploaded font assets with the compiler's font book so
/// `#text(font: "Family Name")` resolves.
///
/// `mapShadow` (in `loadVfs`) only makes an asset readable *by path*; it does
/// not teach the compiler which font family the bytes provide. Typst builds its
/// font book from data handed in at compiler-init time, via `$typst.use(...)`
/// font providers, so we feed each font's raw bytes through
/// `TypstSnippet.preloadFontData`. Fonts are still `mapShadow`-ed alongside every
/// other asset, so one referenced by path keeps working too.
///
/// Registration is idempotent: each font path is remembered so recompiles don't
/// re-add it. Providers can only be attached before the (single, global)
/// compiler is built; a font that first appears afterwards — e.g. uploaded
/// mid-session — cannot be injected into the already-initialised compiler, so we
/// warn instead of throwing, and a reload picks it up.
function registerFonts(assets: TypstBinaryFile[]): void {
  const pending = assets.filter(
    (asset) => isFontAsset(asset.path) && !registeredFontPaths.has(asset.path),
  );
  if (pending.length === 0) return;
  if (compilerBuilt) {
    console.warn(
      `[typst] Font asset(s) added after the compiler was initialised; reload the preview to use them: ${pending
        .map((asset) => asset.path)
        .join(', ')}`,
    );
    return;
  }
  for (const font of pending) {
    $typst.use(TypstSnippet.preloadFontData(font.bytes));
    registeredFontPaths.add(font.path);
  }
}
