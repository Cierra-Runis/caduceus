import { $typst, initOptions } from '@myriaddreamin/typst.ts';

// Client-side Typst compiler (reflexo WASM). This is the single place that
// initialises the compiler/renderer and turns the project's files into preview
// SVG, shared by every consumer (the project preview panel today; LSP/export
// later).
//
// Wasm is pulled from the CDN, version-pinned to the installed JS API so the
// glue and the wasm never skew. The compiler auto-loads Typst's default
// ("text") font assets — New Computer Modern, Libertinus, NCM Math, … —
// which is what documents (and math) fall back to; project fonts are uploaded
// and registered on top (see registerFonts).
const TYPST_VERSION = '0.7.0';

const compilerModule = () =>
  `https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-web-compiler@${TYPST_VERSION}/pkg/typst_ts_web_compiler_bg.wasm`;

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

// Which font set is currently registered, so repeated compiles (every
// keystroke) don't re-register the same fonts.
let registeredFontKey = '';
// Typst's default ("text") fonts, fetched once and reused. They must be
// re-added on every setFonts (which REPLACES the book), or registering an
// uploaded font would wipe the built-ins (New Computer Modern, NCM Math, …).
let defaultFontBytes: null | Promise<Uint8Array[]> = null;

/// Register project fonts into the compiler's font book so `#set text(font:
/// "…")` can select them. Unlike images/data, fonts are not resolved by path —
/// they are looked up by the family name embedded in the font, so the raw bytes
/// go into the book (not the shadow FS). `key` identifies the current font set;
/// registration is skipped when it hasn't changed.
///
/// `compiler.setFonts` REPLACES the whole font book, so the default fonts are
/// re-added alongside the uploaded ones — otherwise selecting an uploaded font
/// would silently drop Typst's built-in New Computer Modern / NCM Math / etc.
export async function registerFonts(
  fonts: Uint8Array[],
  key: string,
): Promise<void> {
  if (key === registeredFontKey) return;
  // Do NOT claim the key until the bytes are actually here. `key` is derived
  // from font *metadata* (available immediately), but the bytes arrive from a
  // later fetch — so the first, empty call must be a no-op that leaves the key
  // unclaimed, or the real registration is skipped as "already done".
  if (fonts.length === 0) return;
  ensureInit();

  // Use the snippet's own resolver — NOT a freshly created one. A fresh
  // `createTypstFontBuilder().init()` re-initializes the shared WASM module,
  // which wipes the compiler's auto-loaded default fonts and its diagnostic
  // state (an unknown font like "Nw" then stops warning). `getFontResolver`
  // reuses the already-initialized module.
  const [compiler, resolver, defaults] = await Promise.all([
    $typst.getCompiler(),
    $typst.getFontResolver(),
    loadDefaultFontBytes(),
  ]);
  // setFonts REPLACES the book, so re-add the defaults alongside the uploaded
  // fonts (otherwise built-ins like New Computer Modern would be dropped).
  for (const bytes of defaults) await resolver.addFontData(bytes);
  for (const bytes of fonts) await resolver.addFontData(bytes);
  await resolver.build(async (built) => compiler.setFonts(built));
  registeredFontKey = key;
}

function abs(path: string): string {
  return `/${path.replace(/^\/+/, '')}`;
}

function ensureInit() {
  if (initialized) return;
  initialized = true;
  $typst.setCompilerInitOptions({ getModule: compilerModule });
  $typst.setRendererInitOptions({
    getModule: () =>
      `https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-renderer@${TYPST_VERSION}/pkg/typst_ts_renderer_bg.wasm`,
  });
}

function loadDefaultFontBytes(): Promise<Uint8Array[]> {
  if (defaultFontBytes) return defaultFontBytes;
  const urls = initOptions._resolveAssets({ assets: ['text'] });
  defaultFontBytes = Promise.all(
    urls.map((url) =>
      fetch(url)
        .then((res) => res.arrayBuffer())
        .then((buf) => new Uint8Array(buf)),
    ),
  );
  return defaultFontBytes;
}
