//! The system textures' lock. A texture's record is 0x78 bytes (0x140 of them): +4 width, +8
//! height, +0xC its pixel kind's description (+4 bytes a pixel, +8 the format), +0x14 its first
//! level's surface, +0x60 locked. Locked, its surface is the one the blitters draw on.

use crate::Invalid;

/// The number of texture records in the pool.
pub const POOL: u32 = 0x140;
/// The fatal errors: locked twice, the lock failed, the unlock failed.
pub const LOCKED_TWICE: u32 = 0x8021_A171;
pub const LOCK_FAILED: u32 = 0x8021_A172;
pub const UNLOCK_FAILED: u32 = 0x8021_A173;

fn in_pool(slot: u32) -> Result<(), Invalid> {
    if slot >= POOL {
        return Err(Invalid("a texture past the pool"));
    }
    Ok(())
}

/// What `d3d::lock_texture` does for a texture slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lock {
    /// No surface: nothing.
    Nothing,
    /// Locked already: fatal error `LOCKED_TWICE`.
    Twice,
    /// The surface's `Lock` (a failure is `LOCK_FAILED`), then locked 1 and the surface's
    /// figures (`surface_info`).
    Lock,
}

/// `d3d::lock_texture`'s choice for texture `slot` (`locked` asked only with a surface).
/// Refuses a slot past the pool.
pub fn lock(slot: u32, surface: u32, locked: impl FnOnce() -> u32) -> Result<Lock, Invalid> {
    in_pool(slot)?;
    Ok(if surface == 0 {
        Lock::Nothing
    } else if locked() == 1 {
        Lock::Twice
    } else {
        Lock::Lock
    })
}

/// What `d3d::unlock_texture` does for a texture slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unlock {
    /// Nothing to do: no surface, or not locked.
    Nothing,
    /// The surface's `Unlock` (a failure is `UNLOCK_FAILED`), locked 0.
    Unlock,
}

/// `d3d::unlock_texture`'s choice for texture `slot` (`locked` asked only with a surface).
/// Refuses a slot past the pool.
pub fn unlock(slot: u32, surface: u32, locked: impl FnOnce() -> u32) -> Result<Unlock, Invalid> {
    in_pool(slot)?;
    if surface == 0 {
        return Ok(Unlock::Nothing);
    }
    match locked() {
        1 => Ok(Unlock::Unlock),
        _ => Ok(Unlock::Nothing),
    }
}

/// The surface the blitters draw on (the drawing surface's dwords from +8), as
/// `d3d::texture_surface_info` sets it for a locked texture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Surface {
    pub width: u32,
    pub height: u32,
    pub bits: u32,
    pub bytes: u32,
    pub row_bytes: u32,
    pub row_dwords: u32,
    /// The pixel kind's format (its description's layout, as the caller names it).
    pub format: u32,
    pub pitch: u32,
    pub pixels: u32,
    /// The bytes between one row's end and the next's start.
    pub gap: u32,
}

/// The surface of a texture `width` x `height` of `bytes` a pixel, locked at `pixels` with
/// `pitch` bytes a row (all wrapping).
pub fn surface_info(
    width: u32,
    height: u32,
    bytes: u32,
    format: u32,
    pitch: u32,
    pixels: u32,
) -> Surface {
    let row = bytes.wrapping_mul(width);
    Surface {
        width,
        height,
        bits: bytes << 3,
        bytes,
        row_bytes: row,
        row_dwords: row >> 2,
        format,
        pitch,
        pixels,
        gap: pitch.wrapping_sub(row),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locks() {
        assert!(lock(POOL, 1, || 0).is_err());
        assert_eq!(lock(0, 0, || unreachable!()), Ok(Lock::Nothing));
        assert_eq!(lock(3, 5, || 1), Ok(Lock::Twice));
        assert_eq!(lock(3, 5, || 2), Ok(Lock::Lock));
        assert!(unlock(POOL, 1, || 0).is_err());
        assert_eq!(unlock(0, 0, || unreachable!()), Ok(Unlock::Nothing));
        assert_eq!(unlock(0, 5, || 0), Ok(Unlock::Nothing));
        assert_eq!(unlock(0, 5, || 2), Ok(Unlock::Nothing));
        assert_eq!(unlock(0, 5, || 1), Ok(Unlock::Unlock));
    }

    #[test]
    fn surfaces() {
        let s = surface_info(0x100, 0x80, 2, 0x1234, 0x240, 0x9000);
        assert_eq!(
            (s.bits, s.row_bytes, s.row_dwords, s.gap),
            (16, 0x200, 0x80, 0x40)
        );
        let s = surface_info(0x10, 1, 2, 0, 0x18, 0);
        assert_eq!((s.row_dwords, s.gap), (8, 0xFFFF_FFF8));
    }
}
