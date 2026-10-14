//! For the audio hooks' self-tests: a copy of an original whose references to game
//! variables point at a test's, its calls at stand-ins, and its jumps out back to where
//! they went.

use crate::framework::{RawHook, game_addr, rva};
use crate::redirect::RedirectedCopy;

pub struct Copy<'a> {
    pub address: usize,
    pub len: usize,
    pub entry: &'a [u8],
    /// Offsets of `call rel32` to relocate.
    pub calls: &'a [usize],
    /// Offsets of the `0F 8x rel32` jumps that leave the function.
    pub jumps_out: &'a [usize],
}

/// The copy, with every dword equal to a game variable's address (an RVA as IDA shows it)
/// replaced by the test's, and the number of replacements checked.
pub fn make(
    hook: &RawHook,
    c: &Copy,
    redirect: &dyn Fn(usize) -> Option<usize>,
    variables: &[(usize, u32)],
    references: usize,
) -> Result<RedirectedCopy, String> {
    let _ = hook;
    let original = game_addr(rva(c.address));
    let copy = unsafe { RedirectedCopy::new(original, c.len, c.entry, c.calls, redirect) }
        .map_err(str::to_string)?;
    for &offset in c.jumps_out {
        unsafe {
            let rel = ((original + offset + 2) as *const i32).read_unaligned();
            let target = (original + offset + 6).wrapping_add(rel as usize);
            copy.retarget(offset + 2, target);
        }
    }
    let mut patched = 0;
    for &(variable, test) in variables {
        let game = game_addr(rva(variable)) as u32;
        for offset in 0..c.len - 3 {
            let at = (copy.addr() + offset) as *const u32;
            if unsafe { at.read_unaligned() } == game {
                unsafe { copy.patch_u32(offset, test) };
                patched += 1;
            }
        }
    }
    if patched != references {
        return Err(format!(
            "{patched} references to the variables, not {references}"
        ));
    }
    Ok(copy)
}
