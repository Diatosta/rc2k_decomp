//! Turns a game-relative path into a full path under one of
//! the two base directories.

use crate::errors::BadInput;

const BUF_LEN: usize = 0x100; // ThreadSlot + 0x10 .. + 0x110

struct Globals {
    mode: u32,
    a: Vec<u8>,
    b: Vec<u8>,
    pct: Vec<u8>,
}

fn globals() -> Result<Globals, BadInput> {
    Ok(Globals {
        mode: crate::files::state::MEDIA_MODE.get(),
        a: crate::files::state::directory_text(0, "the base directory")?,
        // The second base directory: an empty text.
        b: Vec::new(),
        // The locale directory: replaces each '%'
        pct: crate::files::state::directory_text(1, "the locale directory")?,
    })
}

/// `path` written to the buffer the resolved path goes to: the current thread's slot's, else
/// the global one (0x100 into the paths).
fn write_out(path: &[u8]) -> Result<(), BadInput> {
    let what = "the resolved path's buffer";
    match crate::threading::find_thread_slot::current_slot()? {
        None => crate::files::state::PATHS.write_at(0x100, path, what),
        Some(slot) => slot.path.write_at(0, path, what),
    }
}

/// Ok(None) = the path is used as it is. `path` excludes the NUL; the result includes one. A
/// result that does not fit the 0x100-byte buffer is refused.
pub fn resolve_path(
    path: &[u8],
    media_mode: u32,
    base_a: &[u8],
    base_b: &[u8],
    pct: &[u8],
) -> Result<Option<Vec<u8>>, BadInput> {
    let bases = rc2k_core::files::paths::Bases {
        media_mode,
        a: base_a,
        b: base_b,
        percent: pct,
    };
    let Some(out) = rc2k_core::files::paths::resolve(path, &bases) else {
        return Ok(None);
    };
    if out.len() > BUF_LEN {
        return Err(BadInput(
            "resolved path longer than the buffer: the original writes past it",
        ));
    }
    Ok(Some(out))
}

/// The path as the game would open it (`path` without its NUL): the path itself if it is used
/// as it is, else the resolved path, written to this thread's buffer. Without a NUL.
pub fn resolve(path: &[u8]) -> Result<Vec<u8>, BadInput> {
    let g = globals()?;
    let Some(resolved) = resolve_path(path, g.mode, &g.a, &g.b, &g.pct)? else {
        return Ok(path.to_vec());
    };
    write_out(&resolved)?;
    Ok(resolved[..resolved.len() - 1].to_vec())
}

#[cfg(test)]
mod tests {
    /// Another thread (the audio and network loads) resolves the same paths as the game's
    /// thread: the second base directory is read from the files layer, not the installer.
    #[test]
    fn another_thread_resolves_the_same_paths() {
        let _lock = crate::test_support::test_lock();
        let paths: [&[u8]; 4] = [b"files\\d\\gfx\\x.bun", b"var\\ral.cfg", b"%x", b"plain"];
        let here: Vec<_> = paths.iter().map(|p| super::resolve(p)).collect();
        let there: Vec<_> =
            std::thread::spawn(move || paths.iter().map(|p| super::resolve(p)).collect::<Vec<_>>())
                .join()
                .unwrap();
        assert_eq!(here.len(), there.len());
        for (a, b) in here.iter().zip(&there) {
            assert_eq!(a.as_ref().ok(), b.as_ref().ok());
        }
        assert!(here.iter().all(Result::is_ok));
    }
}
