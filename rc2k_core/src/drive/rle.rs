//! The replays' run-length packing (converted 2026-10-21 from `drive::unpack_rle`
//! 0x41F080): a signed count byte; below 0, that many bytes copied; above 0, the next
//! byte repeated count + 1 times; 0 ends it.

use crate::Invalid;
use crate::strings::bytes::Bytes;

/// The most bytes unpacked: past it the packing is refused (the original runs on; a
/// guard against a broken frame).
pub const MOST_OUT: u32 = 0x10_0000;

fn byte(packed: &(impl Bytes + ?Sized), at: u32) -> Result<u8, Invalid> {
    packed
        .byte(at as usize)
        .ok_or(Invalid("a packed run past its data"))
}

/// The bytes the packing unpacks to; refused past `MOST_OUT`.
pub fn unpacked_len(packed: &(impl Bytes + ?Sized)) -> Result<u32, Invalid> {
    let (mut from, mut n) = (0u32, 0u32);
    loop {
        let c = byte(packed, from)? as i8 as i32;
        from += 1;
        if c == 0 {
            return Ok(n);
        }
        if c < 0 {
            n += (-c) as u32;
            from += (-c) as u32;
        } else {
            n += c as u32 + 1;
            from += 1;
        }
        if n > MOST_OUT {
            return Err(Invalid("a packed run past 0x100000 bytes"));
        }
    }
}

/// `drive::unpack_rle`: the packing unpacked, each byte given to `put(offset, byte)` in
/// the original's order (a copy byte by byte forwards, as `rep movsb`); the bytes
/// written. Checked whole before anything is put.
pub fn unpack_rle(
    packed: &(impl Bytes + ?Sized),
    mut put: impl FnMut(u32, u8),
) -> Result<u32, Invalid> {
    unpacked_len(packed)?;
    let (mut from, mut at) = (0u32, 0u32);
    loop {
        let c = byte(packed, from)? as i8 as i32;
        from += 1;
        if c == 0 {
            return Ok(at);
        }
        if c < 0 {
            for _ in 0..-c {
                put(at, byte(packed, from)?);
                from += 1;
                at += 1;
            }
        } else {
            let b = byte(packed, from)?;
            from += 1;
            for _ in 0..=c {
                put(at, b);
                at += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unpacked(p: &[u8]) -> Result<Vec<u8>, Invalid> {
        let mut out = Vec::new();
        let n = unpack_rle(p, |k, b| {
            assert_eq!(k as usize, out.len());
            out.push(b);
        })?;
        assert_eq!(n as usize, out.len());
        Ok(out)
    }

    #[test]
    fn runs() {
        assert_eq!(unpacked(&[0]), Ok(vec![]));
        assert_eq!(
            unpacked(&[0xFD, 1, 2, 3, 1, 0x41, 0]),
            Ok(vec![1, 2, 3, 0x41, 0x41])
        );
        assert_eq!(
            unpacked(&[0x02, 0x11, 0xFF, 0x22, 0]),
            Ok(vec![0x11, 0x11, 0x11, 0x22])
        );
        assert!(unpacked(&[0x02]).is_err());
        let mut big = vec![];
        for _ in 0..0x2001 {
            big.extend([0x7F, 0]);
        }
        big.push(0);
        assert!(unpacked_len(big.as_slice()).is_err());
    }
}
