//! The game's quicksorts of records in its memory (the depth and sort entries, the car's
//! points, the shadow's points) through `rc2k_core::race::quick_sort`: the records from
//! esi to edi (the last, both included) read, sorted, the changed ones written back, the
//! registers set as the original leaves them.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use rc2k_core::race::quick_sort::{Order, quick_sort};

/// The records of `N` dwords from esi to edi sorted by dword `key` in `order`; the
/// registers as the original leaves them (nothing done when esi is not below edi, as
/// its first test). Refuses (before anything) an edi not a whole number of records
/// from esi (the original's two places would never meet) and a sort that would go past
/// the records.
pub unsafe fn sort_records<const N: usize>(
    r: &mut Regs,
    key: usize,
    order: Order,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::quick_sort::quick_sort`.
    if r.esi >= r.edi {
        return Ok(());
    }
    let size = 4 * N as u32;
    let span = r.edi - r.esi;
    if !span.is_multiple_of(size) {
        return Err(BadInput("a sort's last record not whole records from its first").into());
    }
    let base = r.esi;
    let count = (span / size + 1) as usize;
    let at = |n: usize| (base as usize + n * size as usize) as *mut [u32; N];
    let mut list: Vec<[u32; N]> = (0..count)
        .map(|n| unsafe { at(n).read_unaligned() })
        .collect();
    let before = list.clone();
    let ends = quick_sort(&mut list, key, order)?;
    for (n, (now, was)) in list.iter().zip(&before).enumerate() {
        if now != was {
            unsafe { at(n).write_unaligned(*now) };
        }
    }
    let place = |n: i64| base.wrapping_add((n as i32 as u32).wrapping_mul(size));
    r.esi = place(ends.first);
    r.edi = place(ends.last);
    if let Some(s) = ends.scan {
        r.eax = s.pivot;
        r.ebx = s.key;
        r.ecx = place(s.left);
        r.edx = place(s.right);
    }
    if let Some(v) = ends.swapped {
        r.ebp = v;
    }
    Ok(())
}
