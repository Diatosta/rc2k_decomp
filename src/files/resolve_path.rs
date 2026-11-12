//! `files::resolve_path` (0x403206): turns a game-relative path into a full path under one of
//! the two base directories.
//!
//! Takes path. Returns resolved path (a buffer), or the input pointer if the path is used as it is.
//! The output buffer has no bounds check. A result longer than 0x100 bytes is refused, and the

use crate::errors::BadInput;

// ---- game data ----

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
        a: crate::files::state::directory_text(0, "the base directory")?, // g_baseDirectory
        b: crate::files::cstring(
            // The one install store the audio thread's file calls read (atomic, never written).
            &crate::install::owner::INSTALL.on_any_thread().texts.base_b,
            "the second base directory",
        )?,
        // g_localeDirectory: replaces each '%'
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

/// Ok(None) = the path is used as it is. `path` excludes the NUL; the result includes one.
/// A result that does not fit the 0x100-byte buffer is refused: the original writes past it.
pub fn resolve_path(
    path: &[u8],
    media_mode: u32,
    base_a: &[u8],
    base_b: &[u8],
    pct: &[u8],
) -> Result<Option<Vec<u8>>, BadInput> {
    // Converted: `rc2k_core::files::paths::resolve`.
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

/// The path as the game would open it, for Rust callers (`path` without its NUL): the path
/// itself if it is used as it is, else the resolved path, written to this thread's buffer
/// as the original writes it. Without a NUL.
pub fn resolve(path: &[u8]) -> Result<Vec<u8>, BadInput> {
    let g = globals()?;
    let Some(resolved) = resolve_path(path, g.mode, &g.a, &g.b, &g.pct)? else {
        return Ok(path.to_vec());
    };
    write_out(&resolved)?;
    Ok(resolved[..resolved.len() - 1].to_vec())
}
