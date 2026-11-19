//! The numbered instance (`--instance <n>`) whose own `var` and `savedata` folders the game's
//! paths take.

/// The instance's number (above 1) whose own folders the game's paths take.
static INSTANCE: std::sync::OnceLock<u32> = std::sync::OnceLock::new();

/// Makes the paths of instance `number` (above 1) go to its own folders; the first call counts.
pub fn set(number: u32) {
    let _ = INSTANCE.set(number);
}

/// The game path `path` (no NUL) moved into this instance's folders when it names `var` or
/// `savedata` (any case, either separator, relative): `instances\<n>\` first. None for any
/// other path, and for the folder's own copy.
pub fn path(path: &[u8]) -> Option<Vec<u8>> {
    let number = *INSTANCE.get()?;
    redirected(path, number)
}

fn redirected(path: &[u8], number: u32) -> Option<Vec<u8>> {
    let first = path.split(|&b| b == b'\\' || b == b'/').next()?;
    let own = first.eq_ignore_ascii_case(b"var") || first.eq_ignore_ascii_case(b"savedata");
    if !own {
        return None;
    }
    let mut moved = format!("instances\\{number}\\").into_bytes();
    moved.extend_from_slice(path);
    Some(moved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_instance_moves_var_and_savedata_only() {
        assert_eq!(
            redirected(b"var\\ral.cfg", 2).as_deref(),
            Some(&b"instances\\2\\var\\ral.cfg"[..])
        );
        assert_eq!(
            redirected(b"SaveData/replay", 3).as_deref(),
            Some(&b"instances\\3\\SaveData/replay"[..])
        );
        assert_eq!(redirected(b"files\\var\\x", 2), None);
        assert_eq!(redirected(b"variety\\x", 2), None);
        assert_eq!(redirected(b"c:\\var", 2), None);
    }
}
