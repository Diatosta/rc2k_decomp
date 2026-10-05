use crate::framework::{MAX_LOGGED, RawHook, Regs, SHADOW_MODE, hook_asm, log};
use crate::strings::string_table::{blob, entry_count, expand};
use crate::strings::utils::bytes_str;
use std::slice::from_raw_parts;
use std::sync::atomic::{AtomicUsize, Ordering};

static TRAMPOLINE: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "strings::expand_entry",
    0x401FE1 - 0x400000,
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

// eax = id, edi = output cursor -> new cursor in edi. The original also clobbers
// eax/ecx/esi, but its callers save those around the call.
extern "C" fn dispatch(r: &mut Regs) {
    HOOK.stats.calls.fetch_add(1, Ordering::Relaxed);
    let (id, dst) = (r.eax, r.edi as *mut u8);
    let tbl = blob();

    if !SHADOW_MODE.load(Ordering::Relaxed) {
        let mut out = dst;
        unsafe {
            expand(tbl, id, &mut |p, n| {
                std::ptr::copy_nonoverlapping(p, out, n);
                out = out.add(n);
            });
        }
        r.edi = out as u32;
        return;
    }

    // Shadow: Rust collects into a Vec, the original does the real write.
    let mut mine = Vec::new();
    unsafe {
        expand(tbl, id, &mut |p, n| {
            mine.extend_from_slice(from_raw_parts(p, n))
        })
    };

    let mut o = *r;
    unsafe { call_original(&mut o) };
    let orig_len = (o.edi as usize).wrapping_sub(dst as usize);

    let same = orig_len == mine.len() && unsafe { from_raw_parts(dst, orig_len) } == &mine[..];
    if !same {
        let n = HOOK.stats.mismatches.fetch_add(1, Ordering::Relaxed);
        if n < MAX_LOGGED {
            log(&format!(
                "MISMATCH expand_entry(id={id}): original wrote {orig_len} bytes, rust {}",
                mine.len()
            ));
        }
    }
    r.edi = o.edi;
}

pub fn self_test() -> (usize, usize) {
    let tbl = blob();
    if TRAMPOLINE.load(Ordering::Relaxed) == 0 || tbl.is_null() {
        log("expand_entry selftest SKIPPED: original or string table not available");
        return (0, 0);
    }
    let count = entry_count();
    let mut failures = 0;

    // Unhooked, so the original's own recursion runs original code too.
    HOOK.run_unhooked(|| {
        for id in 0..count as u32 {
            let mut mine = Vec::new();
            unsafe {
                expand(tbl, id, &mut |p, n| {
                    mine.extend_from_slice(from_raw_parts(p, n))
                })
            };

            // Slack, so a wrong Rust length can't make the original overflow the buffer.
            let mut buf = vec![0xAAu8; mine.len() + 4096];
            let mut ro = Regs {
                eax: id,
                edi: buf.as_mut_ptr() as u32,
                ..Regs::default()
            };
            unsafe { call_original(&mut ro) };
            let orig_len = (ro.edi as usize).wrapping_sub(buf.as_ptr() as usize);

            if buf.get(..orig_len) != Some(&mine[..]) {
                failures += 1;
                log(&format!(
                    "expand_entry selftest FAILED: id={id} original {} rust {}",
                    bytes_str(buf.get(..orig_len).unwrap_or(&[])),
                    bytes_str(&mine)
                ));
            }
        }
    });
    (count, failures)
}
