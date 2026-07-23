/// Client-side mirror of the server's virtual-filesystem path rules
/// (`server/src/models/path.rs`). Used only for instant, pre-submit feedback in
/// the upload dialog — the server remains the single source of truth and
/// re-validates everything.

const MAX_SEGMENT_LEN = 255;
const MAX_PATH_LEN = 1024;
const MAX_DEPTH = 32;

/// The ancestor directories implied by a path, e.g.
/// `a/b/c.typ` → `['a', 'a/b']`.
export function ancestorDirectories(path: string): string[] {
  const segments = path.split('/');
  const out: string[] = [];
  for (let end = 1; end < segments.length; end++) {
    out.push(segments.slice(0, end).join('/'));
  }
  return out;
}

/// Whether `input` is a legal project-root-relative path: non-empty, within the
/// caps, no empty / `.` / `..` segment, and no segment with an illegal
/// character or leading/trailing whitespace.
export function isValidPath(input: string): boolean {
  const path = input.trim();
  if (path.length === 0 || path.length > MAX_PATH_LEN) return false;

  const segments = path.split('/');
  if (segments.length > MAX_DEPTH) return false;

  for (const segment of segments) {
    if (segment.length === 0 || segment.length > MAX_SEGMENT_LEN) return false;
    if (segment === '.' || segment === '..') return false;
    if (segment !== segment.trim()) return false;
    if (hasIllegalChar(segment)) return false;
  }
  return true;
}

/// A path segment may not contain a backslash or an ASCII control character.
/// Interior spaces are allowed; leading/trailing whitespace is caught by the
/// separate trim check.
function hasIllegalChar(segment: string): boolean {
  for (const ch of segment) {
    const code = ch.charCodeAt(0);
    if (ch === '\\' || code < 0x20 || code === 0x7f) return true;
  }
  return false;
}
