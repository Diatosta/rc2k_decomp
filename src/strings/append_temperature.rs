use super::format_u32;
use super::string_table::append_by_id;
use crate::framework::{MAX_LOGGED, RawHook, Regs, SHADOW_MODE, game_addr, hook_asm, log};
use crate::math::celsius_to_fahrenheit;
use crate::strings::utils::bytes_str;
use std::slice::from_raw_parts;
use std::sync::atomic::{AtomicUsize, Ordering};

static TRAMPOLINE: AtomicUsize = AtomicUsize::new(0);

const UNITS_RVA: usize = 0x51DE2C - 0x400000; // dword_51DE2C: 0 = convert C -> F
const LABEL_IDS_RVA: usize = 0x51EB84 - 0x400000; // word_51EB84[units]: label string ids

pub static HOOK: RawHook = RawHook::new(
    "strings::append_temperature",
    0x401AD9 - 0x400000,
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

fn units() -> u32 {
    unsafe { (game_addr(UNITS_RVA) as *const u32).read_volatile() }
}

/// Returns (bytes written, the eax the original leaves behind).
fn reimplemented(celsius: i32) -> (Vec<u8>, u32) {
    let unit = units();
    let mut v = celsius;
    if unit == 0 {
        v = celsius_to_fahrenheit::reimplemented(v);
    }

    let mut out = Vec::new();
    if v < 0 {
        out.push(b'-');
        v = v.wrapping_neg(); // the `neg eax` before the call to append_minus
    }
    out.extend(format_u32::reimplemented(v as u32, 0));

    let label_id = unsafe {
        (game_addr(LABEL_IDS_RVA) as *const u16)
            .wrapping_add(unit as usize)
            .read_volatile()
    } as u32;
    let eax = unsafe {
        append_by_id(label_id, &mut |p, n| {
            out.extend_from_slice(from_raw_parts(p, n))
        })
    };
    (out, eax)
}

// eax = celsius value, ebx = dst -> new dst in ebx; eax = what append_string_by_id leaves
extern "C" fn dispatch(r: &mut Regs) {
    HOOK.stats.calls.fetch_add(1, Ordering::Relaxed);
    let (value, dst) = (r.eax as i32, r.ebx as *mut u8);
    let (mine, mine_eax) = reimplemented(value);

    if !SHADOW_MODE.load(Ordering::Relaxed) {
        unsafe { std::ptr::copy_nonoverlapping(mine.as_ptr(), dst, mine.len()) };
        r.ebx = (dst as usize + mine.len()) as u32;
        r.eax = mine_eax;
        return;
    }

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
                "MISMATCH append_temperature(value={value}): original wrote {orig_len} bytes (eax={:#x}), rust {} (eax={mine_eax:#x})",
                o.eax,
                mine.len()
            ));
        }
    }
    r.ebx = o.ebx;
    r.eax = o.eax;
}

pub fn self_test() -> (usize, usize) {
    if TRAMPOLINE.load(Ordering::Relaxed) == 0 {
        log("append_temperature selftest SKIPPED: original not available");
        return (0, 0);
    }
    // Tests the unit the game currently has selected: switch it in the game's options
    // and run selftest again to cover the other branch.
    let unit = units();
    let mut inputs: Vec<i32> = (-5000..=5000).collect();
    inputs.extend([-1_000_000_000, -1_000_000, 1_000_000, 1_000_000_000]);
    if unit != 0 {
        // Only without the C -> F conversion: with it, the original's idiv traps beyond ~±1.19e9.
        inputs.extend([i32::MIN, i32::MAX]);
    }

    let mut failures = 0;
    for &c in &inputs {
        let (mine, mine_eax) = reimplemented(c);
        let mut buf = vec![0xAAu8; mine.len() + 4096];
        let mut ro = Regs {
            eax: c as u32,
            ebx: buf.as_mut_ptr() as u32,
            ..Regs::default()
        };
        unsafe { call_original(&mut ro) };
        let orig_len = (ro.ebx as usize).wrapping_sub(buf.as_ptr() as usize);

        if buf.get(..orig_len) != Some(&mine[..]) || ro.eax != mine_eax {
            failures += 1;
            log(&format!(
                "append_temperature selftest FAILED: value={c} units={unit} original {} rust {}",
                bytes_str(buf.get(..orig_len).unwrap_or(&[])),
                bytes_str(&mine)
            ));
        }
    }
    (inputs.len(), failures)
}
