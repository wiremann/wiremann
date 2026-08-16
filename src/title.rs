/// Strips trailing bracketed or parenthesised groups from a track title.
///
/// Files ripped from streaming sites often carry suffixes such as
/// "(Official Music Video)" or "[From: Movie]" which pollute lyric and
/// album-art search queries. This removes them so search APIs get a clean
/// title. Search consumers should try the stripped title first and fall back
/// to the original when it finds nothing.
#[must_use]
pub fn strip_search_suffixes(title: &str) -> &str {
    let trimmed = title.trim_end();

    let bytes = trimmed.as_bytes();
    let mut end = bytes.len();

    loop {
        let Some(&close) = bytes[..end].last() else {
            break;
        };

        let open = match close {
            b')' => b'(',
            b']' => b'[',
            b'}' => b'{',
            _ => break,
        };

        let mut depth = 0i32;
        let mut group_start = None;

        for i in (0..end).rev() {
            let b = bytes[i];
            if b == close {
                depth += 1;
            } else if b == open {
                depth -= 1;
                if depth == 0 {
                    group_start = Some(i);
                    break;
                }
            } else if matches!(b, b'(' | b')' | b'[' | b']' | b'{' | b'}') {
                // A different bracket kind appears before the trailing group
                // closes — treat the title as-is to be safe.
                break;
            }
        }

        let Some(start) = group_start else {
            break;
        };

        if start == 0 {
            // The entire title is one bracketed group; keep it.
            break;
        }

        end = bytes[..start]
            .iter()
            .rposition(|&b| !b.is_ascii_whitespace())
            .map_or(0, |i| i + 1);
    }

    &trimmed[..end]
}

#[cfg(test)]
mod tests {
    use super::strip_search_suffixes;

    #[test]
    fn strips_parenthesized_suffix() {
        assert_eq!(strip_search_suffixes("Song Name (Official Music Video)"), "Song Name");
        assert_eq!(strip_search_suffixes("Song (From 'Movie' Soundtrack)"), "Song");
    }

    #[test]
    fn strips_bracketed_suffix() {
        assert_eq!(strip_search_suffixes("Song Name [Official Video]"), "Song Name");
        assert_eq!(strip_search_suffixes("Song [From: Movie]"), "Song");
    }

    #[test]
    fn strips_multiple_trailing_groups() {
        assert_eq!(
            strip_search_suffixes("Song Name (Official) [Clean]"),
            "Song Name"
        );
    }

    #[test]
    fn strips_featuring_suffix_too() {
        // Trailing paren/bracket groups are stripped even if they look
        // meaningful; the raw title is a fallback if the search fails.
        assert_eq!(strip_search_suffixes("Song (feat. Other)"), "Song");
    }

    #[test]
    fn keeps_plain_title() {
        assert_eq!(strip_search_suffixes("Song Name"), "Song Name");
    }

    #[test]
    fn no_unpaired_bracket() {
        assert_eq!(strip_search_suffixes("Song [Official"), "Song [Official");
    }

    #[test]
    fn trims_whitespace_after_group() {
        assert_eq!(strip_search_suffixes("Song Name  (official)  "), "Song Name");
    }
}
