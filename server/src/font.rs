//! Font detection for uploaded assets.
//!
//! Typst selects fonts by *family name* (`#set text(font: "…")`), not by path,
//! so an uploaded font is useless to the client compiler unless we know the
//! families it provides. We therefore identify fonts at upload time — by their
//! binary signature, **not** their extension (a `.ttf` may be misnamed, and a
//! `.otf`/`.ttc`/`.otc` are just as valid) — and parse out the family names to
//! store alongside the file. The client then registers exactly these bytes into
//! the compiler's font book under those families.
//!
//! Only the sfnt family Typst actually supports is recognized: TrueType
//! (`.ttf`), OpenType/CFF (`.otf`), and their collections (`.ttc`/`.otc`).
//! WOFF/WOFF2 are web-only compressed wrappers Typst cannot use, so they are
//! deliberately *not* treated as fonts.

use ttf_parser::{Face, fonts_in_collection, name_id};

/// If `bytes` are a supported font, return the (deduplicated) family names it
/// provides — possibly empty if the font carries no readable family name but is
/// otherwise valid. `None` means "not a font we can use".
pub fn detect(bytes: &[u8]) -> Option<Vec<String>> {
    if !has_sfnt_magic(bytes) {
        return None;
    }

    // A collection (`ttcf`) holds several faces, each with its own family; a
    // single font is treated as a one-face collection.
    let face_count = fonts_in_collection(bytes).unwrap_or(1);
    let mut families: Vec<String> = Vec::new();
    let mut any_face_parsed = false;

    for index in 0..face_count {
        let Ok(face) = Face::parse(bytes, index) else {
            continue;
        };
        any_face_parsed = true;
        if let Some(family) = family_name(&face)
            && !families.contains(&family)
        {
            families.push(family);
        }
    }

    // Magic matched but nothing parsed → corrupt/unsupported; not usable.
    any_face_parsed.then_some(families)
}

/// The four-byte sfnt signatures Typst can consume. Matched on the raw header
/// so a mislabeled or extensionless upload is still recognized.
fn has_sfnt_magic(bytes: &[u8]) -> bool {
    matches!(
        bytes.first_chunk::<4>(),
        Some(&[0x00, 0x01, 0x00, 0x00]) // TrueType outlines
            | Some(b"OTTO")             // OpenType with CFF
            | Some(b"ttcf")             // TrueType/OpenType collection
            | Some(b"true")             // legacy Apple TrueType
            | Some(b"typ1") // legacy PostScript in sfnt
    )
}

/// Prefer the typographic family (name id 16) and fall back to the legacy
/// family (name id 1) — the same precedence Typst uses.
fn family_name(face: &Face) -> Option<String> {
    let mut legacy = None;
    for name in face.names() {
        match name.name_id {
            name_id::TYPOGRAPHIC_FAMILY => {
                if let Some(value) = name.to_string() {
                    return Some(value);
                }
            }
            name_id::FAMILY if legacy.is_none() => {
                legacy = name.to_string();
            }
            _ => {}
        }
    }
    legacy
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_font_bytes() {
        assert_eq!(detect(b"not a font at all"), None);
        assert_eq!(detect(&[]), None);
        // A PNG header is not a font.
        assert_eq!(detect(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a]), None);
        // Plain UTF-8 text.
        assert_eq!(detect("= Title\nhello".as_bytes()), None);
    }

    #[test]
    fn magic_matches_only_sfnt_signatures() {
        assert!(has_sfnt_magic(&[0x00, 0x01, 0x00, 0x00, 0xAB]));
        assert!(has_sfnt_magic(b"OTTOxxxx"));
        assert!(has_sfnt_magic(b"ttcfxxxx"));
        assert!(has_sfnt_magic(b"truexxxx"));
        assert!(has_sfnt_magic(b"typ1xxxx"));
        assert!(!has_sfnt_magic(b"wOFF")); // WOFF is not a usable font
        assert!(!has_sfnt_magic(b"wOF2")); // WOFF2 likewise
        assert!(!has_sfnt_magic(b"%PDF"));
        assert!(!has_sfnt_magic(&[0x00, 0x01])); // too short
    }

    #[test]
    fn sfnt_magic_but_corrupt_body_is_not_a_font() {
        // Right signature, but the rest is garbage: no face parses, so it is
        // reported as "not a font" rather than a font with no families.
        let mut bytes = vec![0x00, 0x01, 0x00, 0x00];
        bytes.extend_from_slice(&[0xFF; 32]);
        assert_eq!(detect(&bytes), None);
    }
}
