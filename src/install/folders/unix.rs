//! Native Unix folders: absolute paths kept as the system names them (a `/` at the end, no
//! case change), shown whole (the game's font has no `~`), typed whole
//! or with `~` for the home folder; by default the user's data
//! folder (`$XDG_DATA_HOME`, `~/.local/share`) + `rally-championship`, not the home
//! folder's root (owner). The installer's file lists name their files with `\`
//! (`files\d\..`): `files::host_path` reads those as `/`.

use super::{Current, Folders, Room};
use crate::errors::{BadInput, Refused};
use std::path::Path;

pub struct Unix;

/// The folder's name in the user's data folder.
const NAME: &str = "rally-championship";

fn home() -> Option<String> {
    std::env::var("HOME")
        .ok()
        .filter(|h| h.starts_with('/'))
        .map(|h| h.trim_end_matches('/').to_string())
}

/// A typed or recorded path made absolute (`~` the home folder, relative from `cwd`, `.`
/// and `..` resolved), without its last `/`; None for a drive path (`c:..`), a control
/// byte, or `~` with no home folder.
fn absolute(text: &str, cwd: &str, home: Option<&str>) -> Option<String> {
    let text = text.trim();
    let b = text.as_bytes();
    if text.chars().any(char::is_control)
        || (b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':')
    {
        return None;
    }
    let full = if text == "~" {
        home?.to_string()
    } else if let Some(rest) = text.strip_prefix("~/") {
        format!("{}/{rest}", home?)
    } else if text.starts_with('/') {
        text.to_string()
    } else {
        format!("{cwd}/{text}")
    };
    let mut parts: Vec<&str> = Vec::new();
    for seg in full.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            s => parts.push(s),
        }
    }
    Some(format!("/{}", parts.join("/")))
}

impl Folders for Unix {
    const DRIVES: bool = false;

    fn first_folder() -> Option<Vec<u8>> {
        // Without a home folder (none set): the name alone, in the current folder.
        let shown = match (std::env::var("XDG_DATA_HOME"), home()) {
            (Ok(d), _) if d.starts_with('/') => format!("{}/{NAME}", d.trim_end_matches('/')),
            (_, Some(h)) => format!("{h}/.local/share/{NAME}"),
            _ => NAME.to_string(),
        };
        Some(crate::files::ansi_bytes(&shown))
    }

    fn normalised(path: &[u8], current: Current) -> Result<Option<Vec<u8>>, BadInput> {
        let cwd = match current()? {
            Some(c) => crate::files::ansi_text(&c),
            None => return Ok(None),
        };
        let text = crate::files::ansi_text(path);
        let text = if text.trim().is_empty() {
            cwd.clone()
        } else {
            text
        };
        let Some(full) = absolute(&text, cwd.trim_end_matches('/'), home().as_deref()) else {
            return Ok(None);
        };
        let mut out = crate::files::ansi_bytes(&full);
        if out.last() != Some(&b'/') {
            out.push(b'/');
        }
        out.push(0);
        Ok(Some(out))
    }

    fn bad_name(folder: &[u8]) -> Result<bool, BadInput> {
        // A `\` would be read as a separator (`files::host_path`); an empty name is `//`.
        let inner = folder.strip_suffix(b"/").unwrap_or(folder);
        Ok(!folder.starts_with(b"/")
            || folder.iter().any(|&c| c < 0x20 || c == b'\\' || c == 0x7F)
            || inner.windows(2).any(|w| w == b"//"))
    }

    fn make(at: crate::place::BytePlace) -> Result<bool, Refused> {
        let text = at.text("a folder")?;
        let folder = crate::files::host_path(&text);
        let made = std::fs::create_dir_all(&folder);
        if let Err(e) = &made {
            crate::framework::log(&format!("install: {}: {e}", folder.display()));
        }
        Ok(made.is_ok())
    }

    fn room(folder: &[u8]) -> Room {
        let full = crate::files::host_path(folder);
        let mut at: &Path = &full;
        while !at.is_dir() {
            match at.parent() {
                Some(p) => at = p,
                None => return Room::Unreadable,
            }
        }
        match crate::os::free_kb_at(at) {
            Some(kb) => Room::Kb(kb),
            None => Room::Unreadable,
        }
    }

    fn shown(folder: &[u8]) -> Vec<u8> {
        let text = crate::files::ansi_text(folder);
        let full = text.trim_end_matches('/');
        crate::files::ansi_bytes(if full.is_empty() { "/" } else { full })
    }

    fn field_most(_original: u8) -> u8 {
        // A whole path: the original's 124 cut a long home folder's without a word.
        200
    }

    fn field_width(_original: u16) -> u16 {
        // Whole paths (`/home/me/.local/share/..`): nearly the 800-pixel screen.
        760
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_folders() {
        let home = Some("/home/me");
        let abs = |t| absolute(t, "/tmp", home);
        assert_eq!(
            abs("~/Games/Rally/").as_deref(),
            Some("/home/me/Games/Rally")
        );
        assert_eq!(abs("a/../b/./c").as_deref(), Some("/tmp/b/c"));
        assert_eq!(abs("/../x").as_deref(), Some("/x"));
        assert_eq!(abs("c:\\x"), None);
        assert_eq!(absolute("~/x", "/tmp", None), None);
        let mut cwd = || Ok(Some(b"/tmp".to_vec()));
        let n = Unix::normalised(b"/opt/Rally Championship", &mut cwd).unwrap();
        assert_eq!(n.as_deref(), Some(&b"/opt/Rally Championship/\0"[..]));
        let n = Unix::normalised(b"", &mut cwd).unwrap();
        assert_eq!(n.as_deref(), Some(&b"/tmp/\0"[..]));
        assert!(!Unix::bad_name(b"/opt/Rally Championship/").unwrap());
        assert!(Unix::bad_name(b"/opt/a\\b/").unwrap());
        assert!(Unix::bad_name(b"opt/").unwrap());
        assert!(matches!(
            Unix::room(b"/no/such/folder/at/all/"),
            Room::Kb(_)
        ));
    }
}
