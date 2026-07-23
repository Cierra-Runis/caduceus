//! Virtual-filesystem path rules for a project's file tree.
//!
//! A project is a flat set of files keyed by [`path`](super::project::ProjectFile::path)
//! plus a set of explicit directory paths. Paths use `/` as the separator and
//! are always *relative to the project root* (no leading slash). These helpers
//! are the single authority for what a legal path is, so create / rename /
//! upload all reject the same garbage before it ever reaches the database — and
//! they keep paths shaped like a real filesystem's, which is also exactly what
//! a future git-backed store needs (each path maps 1:1 to a tree entry).

use derive_more::Display;

/// Hard caps that keep a single path bounded. Generous enough to never bother a
/// human, tight enough to reject absurd input.
const MAX_SEGMENT_LEN: usize = 255;
const MAX_PATH_LEN: usize = 1024;
const MAX_DEPTH: usize = 32;

#[derive(Debug, Display, PartialEq, Clone)]
pub enum PathError {
    #[display("Path must not be empty")]
    Empty,
    #[display("Path is too long (max {MAX_PATH_LEN} characters)")]
    TooLong,
    #[display("Path is nested too deeply (max {MAX_DEPTH} levels)")]
    TooDeep,
    #[display(
        "Path must be relative and contain no empty segments (no leading, \
         trailing, or repeated '/')"
    )]
    EmptySegment,
    #[display("Path may not contain a '.' or '..' segment")]
    DotSegment,
    #[display("Path segment '{_0}' contains an illegal character")]
    IllegalCharacter(String),
}

/// Validate an arbitrary, user-supplied path and return it in canonical form.
///
/// The canonical form is the same string with surrounding whitespace of the
/// *whole* input trimmed off; internal structure is validated but never
/// silently rewritten — a path that is not already clean is rejected rather
/// than "fixed", so what the user sees in the tree is exactly what is stored.
///
/// Rules (a segment is the text between `/` separators):
/// - non-empty overall, within the length / depth caps;
/// - no empty segment (rejects a leading `/`, trailing `/`, or `//`);
/// - no `.` or `..` segment (no traversal, no ambiguity);
/// - no segment containing `/` (impossible post-split), `\\`, NUL, or any
///   ASCII control character, and no segment that is only whitespace or has
///   leading/trailing whitespace (those read as different files to humans but
///   collide visually).
pub fn normalize_path(input: &str) -> Result<String, PathError> {
    let path = input.trim();
    if path.is_empty() {
        return Err(PathError::Empty);
    }
    if path.len() > MAX_PATH_LEN {
        return Err(PathError::TooLong);
    }

    let segments: Vec<&str> = path.split('/').collect();
    if segments.len() > MAX_DEPTH {
        return Err(PathError::TooDeep);
    }

    for segment in &segments {
        validate_segment(segment)?;
    }

    Ok(path.to_string())
}

/// Validate a directory path with the same rules as a file path. Directories
/// are stored without a trailing slash, exactly like a file path with no
/// extension, so the same normalization applies.
pub fn normalize_directory(input: &str) -> Result<String, PathError> {
    normalize_path(input)
}

fn validate_segment(segment: &str) -> Result<(), PathError> {
    if segment.is_empty() {
        return Err(PathError::EmptySegment);
    }
    if segment == "." || segment == ".." {
        return Err(PathError::DotSegment);
    }
    if segment.len() > MAX_SEGMENT_LEN {
        return Err(PathError::TooLong);
    }
    if segment != segment.trim() {
        return Err(PathError::IllegalCharacter(segment.to_string()));
    }
    if segment
        .chars()
        .any(|c| c == '\\' || c == '\0' || c.is_control())
    {
        return Err(PathError::IllegalCharacter(segment.to_string()));
    }
    Ok(())
}

/// The ancestor directories implied by a file or directory path, deepest last.
/// `chapters/intro/main.typ` implies `chapters` and `chapters/intro`. Used to
/// detect file-vs-directory collisions (a path may not be both).
pub fn ancestor_directories(path: &str) -> Vec<String> {
    let segments: Vec<&str> = path.split('/').collect();
    let mut ancestors = Vec::with_capacity(segments.len().saturating_sub(1));
    for end in 1..segments.len() {
        ancestors.push(segments[..end].join("/"));
    }
    ancestors
}

/// Whether `path` sits inside directory `dir` (at any depth). A directory does
/// not contain itself.
pub fn is_within(path: &str, dir: &str) -> bool {
    path.len() > dir.len() && path.starts_with(dir) && path.as_bytes()[dir.len()] == b'/'
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn accepts_simple_and_nested_paths() {
        assert_eq!(normalize_path("main.typ").unwrap(), "main.typ");
        assert_eq!(
            normalize_path("chapters/intro.typ").unwrap(),
            "chapters/intro.typ"
        );
        // Surrounding whitespace of the whole input is trimmed.
        assert_eq!(normalize_path("  main.typ  ").unwrap(), "main.typ");
    }

    #[test]
    fn rejects_empty_and_whitespace() {
        assert_eq!(normalize_path(""), Err(PathError::Empty));
        assert_eq!(normalize_path("   "), Err(PathError::Empty));
    }

    #[test]
    fn rejects_empty_segments() {
        assert_eq!(normalize_path("/main.typ"), Err(PathError::EmptySegment));
        assert_eq!(normalize_path("main.typ/"), Err(PathError::EmptySegment));
        assert_eq!(normalize_path("a//b.typ"), Err(PathError::EmptySegment));
    }

    #[test]
    fn rejects_dot_segments() {
        assert_eq!(normalize_path("./main.typ"), Err(PathError::DotSegment));
        assert_eq!(normalize_path("../secret"), Err(PathError::DotSegment));
        assert_eq!(normalize_path("a/../b"), Err(PathError::DotSegment));
    }

    #[test]
    fn rejects_illegal_characters() {
        assert!(matches!(
            normalize_path("a\\b"),
            Err(PathError::IllegalCharacter(_))
        ));
        assert!(matches!(
            normalize_path("a\tb"),
            Err(PathError::IllegalCharacter(_))
        ));
        // Interior segment with surrounding whitespace.
        assert!(matches!(
            normalize_path("a/ b /c"),
            Err(PathError::IllegalCharacter(_))
        ));
    }

    #[test]
    fn rejects_too_deep() {
        let deep = (0..=MAX_DEPTH).map(|_| "a").collect::<Vec<_>>().join("/");
        assert_eq!(normalize_path(&deep), Err(PathError::TooDeep));
    }

    #[test]
    fn rejects_too_long_segment_and_path() {
        let long_segment = "a".repeat(MAX_SEGMENT_LEN + 1);
        assert_eq!(normalize_path(&long_segment), Err(PathError::TooLong));
    }

    #[test]
    fn computes_ancestor_directories() {
        assert_eq!(ancestor_directories("main.typ"), Vec::<String>::new());
        assert_eq!(
            ancestor_directories("chapters/intro/main.typ"),
            vec!["chapters".to_string(), "chapters/intro".to_string()]
        );
    }

    #[test]
    fn is_within_matches_only_true_descendants() {
        assert!(is_within("a/b.typ", "a"));
        assert!(is_within("a/b/c.typ", "a/b"));
        assert!(!is_within("a", "a"));
        assert!(!is_within("ab/c.typ", "a"));
        assert!(!is_within("a.typ", "a"));
    }
}
