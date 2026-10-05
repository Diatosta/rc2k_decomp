use crate::framework::{MAX_LOGGED, RawHook, Regs, SHADOW_MODE, hook_asm, log};
use crate::strings::string_table::{LOW_IDS, append_by_id, blob, entry_count, ptr_low};
use crate::strings::utils::bytes_str;
use std::ops::Range;
use std::slice::from_raw_parts;
use std::sync::atomic::{AtomicUsize, Ordering};

static TRAMPOLINE: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "strings::append_string_by_id",
    0x401FA3 - 0x400000,
    thunk,
    &TRAMPOLINE,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = TRAMPOLINE
);

// eax = id, ebx = dst -> new dst in ebx; eax = the string pointer (pointer-table branches)
// or the id itself (fallback), exactly as the original leaves it.
extern "C" fn dispatch(r: &mut Regs) {
    HOOK.stats.calls.fetch_add(1, Ordering::Relaxed);
    let (id, dst) = (r.eax, r.ebx as *mut u8);

    if !SHADOW_MODE.load(Ordering::Relaxed) {
        let mut out = dst;
        let eax = unsafe {
            append_by_id(id, &mut |p, n| {
                std::ptr::copy_nonoverlapping(p, out, n);
                out = out.add(n);
            })
        };
        r.ebx = out as u32;
        r.eax = eax;
        return;
    }

    let mut mine = Vec::new();
    let mine_eax =
        unsafe { append_by_id(id, &mut |p, n| mine.extend_from_slice(from_raw_parts(p, n))) };

    let mut o = *r;
    unsafe { call_original(&mut o) };
    let orig_len = (o.ebx as usize).wrapping_sub(dst as usize);

    let same = orig_len == mine.len()
        && unsafe { from_raw_parts(dst, orig_len) } == &mine[..]
        && o.eax == mine_eax;
    if !same {
        let n = HOOK.stats.mismatches.fetch_add(1, Ordering::Relaxed);
        if n < MAX_LOGGED {
            log(&format!(
                "MISMATCH append_string_by_id(id={id:#x}): original wrote {orig_len} bytes (eax={:#x}), rust {} (eax={mine_eax:#x})",
                o.eax,
                mine.len()
            ));
        }
    }
    r.ebx = o.ebx;
    r.eax = o.eax;
}

/// Ids to test in the 0x1000 branch. Fill this in once you know how long dword_525DE8 is
/// (check the array in IDA), e.g. 0x1000..0x1100. Empty = that branch isn't covered here.
const HIGH_IDS: Range<u32> = 0x1000..0x1000;

pub fn self_test() -> (usize, usize) {
    if TRAMPOLINE.load(Ordering::Relaxed) == 0 || blob().is_null() {
        log("append_string_by_id selftest SKIPPED: original or string table not available");
        return (0, 0);
    }
    let entries = entry_count() as u32;
    // Low range: ids whose src[] pointer is set (direct branch), or that fall through
    // to expand_entry and are valid entries.
    let ids: Vec<u32> = HIGH_IDS
        .chain(LOW_IDS.filter(|&id| unsafe { !ptr_low(id).is_null() } || id < entries))
        .collect();

    let mut failures = 0;
    for &id in &ids {
        let mut mine = Vec::new();
        let mine_eax =
            unsafe { append_by_id(id, &mut |p, n| mine.extend_from_slice(from_raw_parts(p, n))) };

        let mut buf = vec![0xAAu8; mine.len() + 4096];
        let mut ro = Regs {
            eax: id,
            ebx: buf.as_mut_ptr() as u32,
            ..Regs::default()
        };
        unsafe { call_original(&mut ro) };
        let orig_len = (ro.ebx as usize).wrapping_sub(buf.as_ptr() as usize);

        if buf.get(..orig_len) != Some(&mine[..]) || ro.eax != mine_eax {
            failures += 1;
            log(&format!(
                "append_string_by_id selftest FAILED: id={id:#x} original {} rust {}",
                bytes_str(buf.get(..orig_len).unwrap_or(&[])),
                bytes_str(&mine)
            ));
        }
    }
    (ids.len(), failures)
}
