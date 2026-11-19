//! A file search's pattern as `FindFirstFileA` matched it (on std):
//! letters without regard to case, `*` any run of characters, `?` one character, or none
//! where the name has a `.` or its end (DOS's rule, so `???.pcx` takes `7.pcx`), and a
//! pattern ending in `.*` takes names without a dot too (`*.*` is `*`).

/// Whether `name` (one file's name, no folder) matches `pattern`. Both are the game's bytes;
/// letters compare as Windows-1252's upper case does for ASCII and Latin-1.
pub fn matches(pattern: &[u8], name: &[u8]) -> bool {
    if let Some(stem) = pattern.strip_suffix(b".*")
        && !name.contains(&b'.')
        && matches(stem, name)
    {
        return true;
    }
    matches_from(pattern, name)
}

fn upper(b: u8) -> u8 {
    match b {
        b'a'..=b'z' | 0xE0..=0xFE if b != 0xF7 => b - 0x20,
        // š œ ž ÿ: their capitals are in Windows-1252's 0x80 .. 0x9F.
        0x9A | 0x9C | 0x9E => b - 0x10,
        0xFF => 0x9F,
        _ => b,
    }
}

fn matches_from(pattern: &[u8], name: &[u8]) -> bool {
    match pattern.split_first() {
        None => name.is_empty(),
        Some((b'*', rest)) => (0..=name.len()).any(|n| matches_from(rest, &name[n..])),
        Some((b'?', rest)) => {
            let none_here = name.first().is_none_or(|&c| c == b'.');
            (none_here && matches_from(rest, name))
                || name
                    .split_first()
                    .is_some_and(|(_, n)| matches_from(rest, n))
        }
        Some((&p, rest)) => name
            .split_first()
            .is_some_and(|(&c, n)| upper(c) == upper(p) && matches_from(rest, n)),
    }
}

/// Names in the order NTFS lists them (by their upper case), which the game's searches met;
/// a folder's own order (Wine's, Linux's) is not kept.
pub fn sort_names(names: &mut [Vec<u8>]) {
    names.sort_by(|a, b| a.iter().map(|&c| upper(c)).cmp(b.iter().map(|&c| upper(c))));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stars_and_letters() {
        assert!(matches(b"*", b"a.b"));
        assert!(matches(b"*.pcx", b"SHOT.PCX"));
        assert!(!matches(b"*.pcx", b"shot.pc"));
        assert!(matches(b"*.*", b"readme"));
        assert!(matches(b"ral.*", b"RAL"));
        assert!(matches(b"Ral.Cfg", b"ral.cfg"));
        assert!(!matches(b"ral.cfg", b"ral.cfg2"));
        assert!(matches(b"\xe7*", b"\xc7a"));
    }

    #[test]
    fn question_marks_are_one_or_none_before_a_dot() {
        assert!(matches(b"???.pcx", b"000.pcx"));
        assert!(matches(b"???.pcx", b"7.pcx"));
        assert!(!matches(b"???.pcx", b"0000.pcx"));
        assert!(matches(b"a?", b"a"));
        assert!(!matches(b"a?b", b"ab"));
    }

    #[test]
    fn names_sort_as_ntfs() {
        let mut names = vec![b"b".to_vec(), b"A".to_vec(), b"a1".to_vec()];
        sort_names(&mut names);
        assert_eq!(names, [b"A".to_vec(), b"a1".to_vec(), b"b".to_vec()]);
    }
}
