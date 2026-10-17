//! For self-tests of functions whose variables are the game's all through a frame (the
//! work copy `g_car` and its neighbours): a `Mirror` of some of the game's ranges, filled
//! from the game's own contents (so that constants have their values), and copies of the
//! originals whose every reference into those ranges is pointed at the mirror (the number
//! of references checked). The Rust cores reach their variables through `Mirror::a`, which
//! for `Mirror::game()` is the game's own address.

use crate::framework::{game_addr, rva};
use crate::redirect::RedirectedCopy;
use crate::service::harness::Region;

/// One original as a copy: (IDA address, length, entry bytes, call offsets, references
/// into the mirrored ranges).
pub type Spec = (u32, usize, &'static [u8], &'static [usize], usize);

/// The game's ranges, or a test's copies of them.
pub struct Mirror {
    ranges: Vec<(u32, u32)>,
    blocks: Vec<Vec<u8>>,
    bases: Vec<u32>,
}

impl Mirror {
    /// The game's own variables.
    pub fn game() -> Self {
        Self {
            ranges: Vec::new(),
            blocks: Vec::new(),
            bases: Vec::new(),
        }
    }

    /// Copies of `ranges` (IDA addresses, end exclusive), filled from the game's.
    pub fn of(ranges: &[(u32, u32)]) -> Self {
        let mut blocks: Vec<Vec<u8>> = ranges
            .iter()
            .map(|&(lo, hi)| {
                let at = game_addr(rva(lo as usize)) as *const u8;
                unsafe { std::slice::from_raw_parts(at, (hi - lo) as usize) }.to_vec()
            })
            .collect();
        let bases = blocks.iter_mut().map(|b| b.as_mut_ptr() as u32).collect();
        Self {
            ranges: ranges.to_vec(),
            blocks,
            bases,
        }
    }

    /// The address of the variable IDA shows at `ida`.
    pub fn a(&self, ida: u32) -> u32 {
        for (k, &(lo, hi)) in self.ranges.iter().enumerate() {
            if (lo..hi).contains(&ida) {
                return self.bases[k] + (ida - lo);
            }
        }
        game_addr(rva(ida as usize)) as u32
    }

    pub unsafe fn get(&self, ida: u32) -> u32 {
        unsafe { (self.a(ida) as *const u32).read_unaligned() }
    }

    pub unsafe fn put(&self, ida: u32, v: u32) {
        unsafe { (self.a(ida) as *mut u32).write_unaligned(v) }
    }

    /// The mirrored ranges, for comparing.
    pub fn regions(&self) -> Vec<Region> {
        self.blocks
            .iter()
            .map(|b| (b.as_ptr() as u32, b.len()))
            .collect()
    }

    /// A copy of one original pointed at the mirror, its calls sent by `redirect` (IDA
    /// addresses in and out).
    pub fn copy(
        &self,
        spec: Spec,
        redirect: &dyn Fn(usize) -> Option<usize>,
    ) -> Result<RedirectedCopy, String> {
        let (address, len, entry, calls, references) = spec;
        let original = game_addr(rva(address as usize));
        let copy = unsafe {
            RedirectedCopy::new(original, len, entry, calls, &|r| {
                redirect(r + crate::framework::IMAGE_BASE)
            })
        }
        .map_err(str::to_string)?;
        let mut patched = 0;
        for offset in 0..len - 3 {
            let at = (copy.addr() + offset) as *const u32;
            let v = unsafe { at.read_unaligned() };
            if self.ranges.iter().any(|&(lo, hi)| (lo..hi).contains(&v)) {
                unsafe { copy.patch_u32(offset, self.a(v)) };
                patched += 1;
            }
        }
        if patched != references {
            return Err(format!(
                "{address:#x}: {patched} references to the mirrored ranges, not {references}"
            ));
        }
        Ok(copy)
    }
}
