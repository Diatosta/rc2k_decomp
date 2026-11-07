//! `files::resolve_path` (0x403206): turns a game-relative path into a full path under one of
//! the two base directories.
//!
//! In:  eax = path
//! Out: eax = resolved path (a buffer), or the input pointer if the path is used as it is;
//!      everything else preserved
//! The output buffer has no bounds check. A result longer than 0x100 bytes is refused, and the
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::pointers::GamePtr;

// ---- game data ----

const BUF_LEN: usize = 0x100; // ThreadSlot + 0x10 .. + 0x110

struct Globals {
    mode: u32,
    a: Vec<u8>,
    b: Vec<u8>,
    pct: Vec<u8>,
}

fn text_at(at: GamePtr, what: &str) -> Result<Vec<u8>, BadInput> {
    crate::files::cstring_at(at, what)
}

fn globals() -> Result<Globals, BadInput> {
    Ok(Globals {
        mode: crate::files::state::MEDIA_MODE.get(),
        a: text_at(
            crate::files::state::base_directory_place(),
            "the base directory",
        )?, // g_baseDirectory
        b: text_at(
            GamePtr::to(&crate::install::owner::install().texts.base_b),
            "the second base directory",
        )?,
        pct: text_at(
            crate::files::state::locale_directory_place(),
            "the locale directory",
        )?, // g_localeDirectory: replaces each '%'
    })
}

/// The buffer the resolved path is written to: the current thread's slot's, else the global one.
fn out_buffer() -> Result<GamePtr, BadInput> {
    Ok(match crate::threading::find_thread_slot::current_slot()? {
        // The global buffer's path text (bytes) starts 0x100 into the paths.
        None => GamePtr::to(&crate::files::state::PATHS).offset(0x100),
        Some(slot) => GamePtr::to(&slot.path),
    })
}

// ---- Rust version ----

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
    out_buffer()?.write_bytes(&resolved, "the resolved path's buffer")?;
    Ok(resolved[..resolved.len() - 1].to_vec())
}
