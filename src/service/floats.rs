//! Comparing results that hold floats (2026-10-20, the conversion to plain floats): the
//! original computes on the x87 at the precision its control word sets, the converted
//! Rust in `f64` where the x87 keeps a value on its stack and `f32` where it stores one.
//! The game runs at double precision most of the time (0x027F) and at extended (0x037F)
//! after its `finit`: at double the two agree bit for bit, at extended the last bit may
//! differ, and an integer made from a float may land one off at a rounding edge. A place
//! declared here that differs within its tolerance is a drift, logged as "DRIFT" (ten
//! lines a name) and counted in `stats`, not a mismatch; anything else that differs is
//! still a mismatch. Rust callers in modules not yet converted call the exact x87
//! versions (the converted ones are `*_plain`), so their own exact comparisons hold.

use crate::errors::Refused;
use crate::framework::{GameHook, RawHook, Regs, SelfTest, log, shadow};
use crate::screens::harness::{CallOriginal, Left};
use crate::service::harness::{Region, Rust, capture, diff_summary, restore, shadow_due};
use std::collections::HashMap;
use std::sync::Mutex;

/// How far a float place may be off.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tolerance {
    /// An `f32`: this many units in the last place.
    Ulps(u32),
    /// An `f32`: this much of the larger magnitude (approximations, `rsqrtss`).
    Relative(f32),
    /// An `i32` made from a float: this much either way.
    Rounded(u32),
    /// An `f32`: this far either way (a difference of near values, where a last-bit change
    /// of the two is large against the result).
    Absolute(f32),
    /// An 80-bit value (ten bytes from the place's start), compared as `f64`s: this much
    /// of the larger magnitude, or this far either way (a sum that nearly cancels).
    Extended(f64, f64),
    /// An `f32`: this many units in the last place, or this far either way (a sum of
    /// products that nearly cancels: its error is the terms' size, not the result's).
    Near(u32, f32),
    /// The same bits, or both NaNs whatever their sign and payload (owner, 2026-10-21: the
    /// x87 keeps the NaN with the larger significand where two meet, Rust's floats promise
    /// no NaN's bits; the game makes no NaN in play). What a place is held to at the
    /// game's double precision.
    Exact,
    /// `Exact` for an 80-bit place.
    ExactExtended,
}

impl Tolerance {
    /// What a place with this tolerance is held to where the two must agree bit for bit.
    pub fn exact(self) -> Tolerance {
        match self {
            Tolerance::Extended(..) | Tolerance::ExactExtended => Tolerance::ExactExtended,
            Tolerance::Rounded(_) => Tolerance::Rounded(0),
            _ => Tolerance::Exact,
        }
    }
}

/// What the converted functions are held to at the x87's single and extended precision.
pub const FLOAT: Tolerance = Tolerance::Ulps(8);
/// For a sum of products of a matrix's size: `FLOAT`, or 2^-20 either way where the sum
/// nearly cancels (seen 2026-10-21, `scene::multiply_matrices` at 0x037F: 9 units off on
/// a result of about 1.8e-12 from terms of about 1; an `f64`'s error on such terms is
/// about 2^-52, the game's matrices hold places up to about 2^20).
pub const SUM: Tolerance = Tolerance::Near(8, 1.0 / (1u32 << 20) as f32);
/// For a radius times a sine or cosine (`fsincos` against `f64`'s): `FLOAT`, or 2^-20
/// either way where the angle is near a multiple of pi / 2 and the result near 0 (seen
/// 2026-10-21, `marks::set_up_squares` / `set_up_hexagons` at 0x027F: 7.7e-14 against
/// 7.7e-14 plus 377 units, the x87's 64-bit pi against `f64`'s).
pub const TRIG: Tolerance = Tolerance::Near(8, 1.0 / (1u32 << 20) as f32);
pub const ROUNDED: Tolerance = Tolerance::Rounded(1);
/// For an 80-bit result made at extended precision and compared with an `f64`'s: the
/// rounding of each step (64 bits against 53) adds up to about 2^-49 over a sum of
/// products (seen 2026-10-20, `rotate_point`, `dot_product3`); 2^-40, far under a float's
/// 2^-24, which is what the game makes of these values.
/// The absolute floor 2^-26: a sum of terms of a stage's size that cancels keeps their
/// last bits' error, an `f64`'s at the terms' size (2^-30 at first; seen 2026-10-20,
/// `rotate_point` at 0x037F: 2.8e-9 and 1.4e-9 on results of about 280 and 200, as an
/// `f64`'s last place at about 1e7, 1.9e-9). Still far under a float's last place there
/// (about 1.5e-5), which is what the game stores.
pub const EXTENDED: Tolerance =
    Tolerance::Extended(1.0 / (1u64 << 40) as f64, 1.0 / (1u64 << 26) as f64);
/// An angle a search of halvings found (a float kept at 80 bits): a drifted step may
/// send the search the other way, by the last steps' size.
pub const ANGLE: Tolerance = Tolerance::Extended(1.0 / 1024.0, 0.0);
/// A code that may be any (a compare's status after a search whose steps drifted).
pub const ANY: Tolerance = Tolerance::Rounded(u32::MAX);

/// The float places of a comparison: in memory (each a run of dwords) and in registers.
#[derive(Clone, Copy, Default)]
pub struct Floats<'a> {
    pub memory: &'a [(Region, Tolerance)],
    pub regs: &'a [(u8, Tolerance)],
}

impl Floats<'_> {
    /// Nothing may differ: the double-precision cases.
    pub const EXACT: Floats<'static> = Floats {
        memory: &[],
        regs: &[],
    };
}

/// The order of an `f32`'s bits as numbers: neighbours one apart.
fn ordered(bits: u32) -> i64 {
    if bits & 0x8000_0000 != 0 {
        -((bits & 0x7FFF_FFFF) as i64)
    } else {
        bits as i64
    }
}

/// Whether two dwords agree within a tolerance.
pub fn close(theirs: u32, mine: u32, tolerance: Tolerance) -> bool {
    if theirs == mine {
        return true;
    }
    match tolerance {
        Tolerance::Ulps(n) => {
            let (a, b) = (f32::from_bits(theirs), f32::from_bits(mine));
            if a.is_nan() || b.is_nan() {
                return a.is_nan() && b.is_nan();
            }
            (ordered(theirs) - ordered(mine)).unsigned_abs() <= n as u64
        }
        Tolerance::Relative(r) => {
            let (a, b) = (f32::from_bits(theirs), f32::from_bits(mine));
            if !a.is_finite() || !b.is_finite() {
                return false;
            }
            (a - b).abs() <= r * a.abs().max(b.abs())
        }
        Tolerance::Rounded(n) => (theirs as i32).abs_diff(mine as i32) <= n,
        Tolerance::Absolute(d) => {
            let (a, b) = (f32::from_bits(theirs), f32::from_bits(mine));
            if a.is_nan() || b.is_nan() {
                return a.is_nan() && b.is_nan();
            }
            (a - b).abs() <= d
        }
        Tolerance::Near(n, d) => {
            let (a, b) = (f32::from_bits(theirs), f32::from_bits(mine));
            if a.is_nan() || b.is_nan() {
                return a.is_nan() && b.is_nan();
            }
            (ordered(theirs) - ordered(mine)).unsigned_abs() <= n as u64
                || (a.is_finite() && b.is_finite() && (a - b).abs() <= d)
        }
        Tolerance::Exact => f32::from_bits(theirs).is_nan() && f32::from_bits(mine).is_nan(),
        Tolerance::Extended(..) | Tolerance::ExactExtended => false,
    }
}

/// Whether ten bytes are an 80-bit NaN (all exponent bits set, a fraction below the
/// integer bit).
fn extended_nan(v: [u8; 10]) -> bool {
    let exponent = u16::from_le_bytes([v[8], v[9]]) & 0x7FFF;
    let fraction = u64::from_le_bytes(v[..8].try_into().unwrap()) << 1;
    exponent == 0x7FFF && fraction != 0
}

/// Two 80-bit values within `relative` of the larger magnitude, or within `absolute`.
pub fn close_extended(theirs: [u8; 10], mine: [u8; 10], relative: f64, absolute: f64) -> bool {
    use rc2k_core::math::extended::to_f64;
    if theirs == mine {
        return true;
    }
    let (a, b) = (to_f64(theirs), to_f64(mine));
    if a.is_nan() || b.is_nan() {
        return a.is_nan() && b.is_nan();
    }
    if !a.is_finite() || !b.is_finite() {
        return false;
    }
    let d = (a - b).abs();
    d <= relative * a.abs().max(b.abs()) || d <= absolute
}

/// Runs `case` at each of the tests' control words with what it is held to there: exact
/// at the game's double precision (`Tolerance::exact`: a NaN matches any NaN), `tolerant`
/// at extended.
pub fn at_test_precisions(tolerant: &Floats, case: &mut dyn FnMut(u16, &Floats)) {
    use crate::math::x87_run::{DOUBLE, TEST_CONTROL_WORDS, with_control_word};
    let memory: Vec<_> = tolerant
        .memory
        .iter()
        .map(|&(r, t)| (r, t.exact()))
        .collect();
    let regs: Vec<_> = tolerant.regs.iter().map(|&(r, t)| (r, t.exact())).collect();
    let exact = Floats {
        memory: &memory,
        regs: &regs,
    };
    for word in TEST_CONTROL_WORDS {
        let floats = if word == DOUBLE { &exact } else { tolerant };
        with_control_word(word, &mut || case(word, floats));
    }
}

/// How two results compare.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Agreement {
    Same,
    /// Different only within the tolerances: so many places.
    Drift(usize),
    Differ,
}

impl Agreement {
    fn and(self, other: Agreement) -> Agreement {
        match (self, other) {
            (Agreement::Differ, _) | (_, Agreement::Differ) => Agreement::Differ,
            (Agreement::Drift(a), Agreement::Drift(b)) => Agreement::Drift(a + b),
            (Agreement::Drift(a), _) | (_, Agreement::Drift(a)) => Agreement::Drift(a),
            _ => Agreement::Same,
        }
    }
}

/// Two captures of `regions` compared, the float places within their tolerances.
pub fn compare_memory(
    regions: &[Region],
    theirs: &[u8],
    mine: &[u8],
    floats: &Floats,
) -> Agreement {
    if theirs.len() != mine.len() {
        return Agreement::Differ;
    }
    let mut drifts = 0;
    let mut start = 0;
    for &(address, len) in regions {
        let (a, b) = (&theirs[start..start + len], &mine[start..start + len]);
        start += len;
        let mut n = 0;
        while n < len {
            if a[n] == b[n] {
                n += 1;
                continue;
            }
            let place = address.wrapping_add(n as u32);
            let found = floats
                .memory
                .iter()
                .find(|((at, bytes), _)| place.wrapping_sub(*at) < *bytes as u32);
            let Some(&((at, _), tolerance)) = found else {
                return Agreement::Differ;
            };
            if let Tolerance::Extended(..) | Tolerance::ExactExtended = tolerance {
                // The ten bytes the place is in, which must lie in this capture region.
                let start = place - (place - at) % 10;
                let offset = start.wrapping_sub(address) as usize;
                if start < address || offset + 10 > len {
                    return Agreement::Differ;
                }
                let value = |s: &[u8]| -> [u8; 10] { s[offset..offset + 10].try_into().unwrap() };
                let (x, y) = (value(a), value(b));
                let agree = match tolerance {
                    Tolerance::Extended(relative, absolute) => {
                        close_extended(x, y, relative, absolute)
                    }
                    _ => x == y || (extended_nan(x) && extended_nan(y)),
                };
                if !agree {
                    return Agreement::Differ;
                }
                drifts += 1;
                n = offset + 10;
                continue;
            }
            // The dword the place is in, which must lie in this capture region.
            let dword = place - (place - at) % 4;
            let offset = dword.wrapping_sub(address) as usize;
            if dword < address || offset + 4 > len {
                return Agreement::Differ;
            }
            let word = |s: &[u8]| u32::from_le_bytes(s[offset..offset + 4].try_into().unwrap());
            if !close(word(a), word(b), tolerance) {
                return Agreement::Differ;
            }
            drifts += 1;
            n = offset + 4;
        }
    }
    if drifts == 0 {
        Agreement::Same
    } else {
        Agreement::Drift(drifts)
    }
}

/// What the Rust version left in the registers against what the original did, the float
/// registers within their tolerances.
pub fn compare_regs(left: &Left, before: &Regs, after: &Regs, floats: &Floats) -> Agreement {
    let mut drifts = 0;
    let mut expected = left.clone();
    for &(reg, value) in &left.regs {
        let theirs = crate::screens::harness::reg_value(after, reg);
        if theirs == value {
            continue;
        }
        match floats.regs.iter().find(|&&(r, _)| r == reg) {
            Some(&(_, tolerance)) if close(theirs, value, tolerance) => {
                drifts += 1;
                expected = expected.with(reg, theirs);
            }
            _ => return Agreement::Differ,
        }
    }
    if !expected.matches(before, after) {
        return Agreement::Differ;
    }
    if drifts == 0 {
        Agreement::Same
    } else {
        Agreement::Drift(drifts)
    }
}

/// The x87's control word now (`fnstcw`).
pub fn control_word() -> u16 {
    let mut word = 0u16;
    unsafe {
        core::arch::asm!("fnstcw word ptr [{w}]", w = in(reg) &raw mut word, options(nostack));
    }
    word
}

/// Logs the x87's control word on a hook's call when it is not the one seen last: the
/// precision the game computes at (0x027F double, 0x037F extended, 0x007F single).
pub fn note_control_word(name: &str) {
    use std::sync::atomic::{AtomicU32, Ordering};
    static LAST: AtomicU32 = AtomicU32::new(u32::MAX);
    let word = control_word() as u32;
    if LAST.swap(word, Ordering::Relaxed) != word {
        log(&format!("x87 control word {word:#06x} on a call of {name}"));
    }
}

/// Drifts so far, by hook.
static DRIFTS: Mutex<Option<HashMap<&'static str, u64>>> = Mutex::new(None);
const DRIFT_LINES: u64 = 10;

/// A drift noted: the first ten of each name logged.
pub fn note_drift(name: &'static str, detail: impl FnOnce() -> String) {
    let count = {
        let mut drifts = DRIFTS.lock().unwrap();
        let count = drifts
            .get_or_insert_with(HashMap::new)
            .entry(name)
            .or_insert(0);
        *count += 1;
        *count
    };
    if count <= DRIFT_LINES {
        log(&format!("DRIFT {name}: {}", detail()));
    }
}

/// The drifts noted, by name (for `stats`).
pub fn drift_counts() -> Vec<(&'static str, u64)> {
    let drifts = DRIFTS.lock().unwrap();
    let mut counts: Vec<_> = drifts
        .iter()
        .flat_map(|d| d.iter().map(|(&name, &n)| (name, n)))
        .collect();
    counts.sort();
    counts
}

/// `service::harness::dispatch_state` with float places.
pub fn dispatch_state_floats(
    hook: &'static RawHook,
    r: &mut Regs,
    call_original: CallOriginal,
    regions: &[Region],
    floats: &Floats,
    rust: Rust,
) {
    hook.count_call();
    if !shadow() {
        match rust(r) {
            Ok(left) => left.apply(r),
            Err(e) => unsafe { crate::fatal::raise::refused(hook, e, r, call_original) },
        }
        return;
    }
    if !shadow_due(hook) {
        unsafe { call_original(r) };
        return;
    }
    let word = crate::math::x87_run::control_word();
    let before = capture(regions);
    let left = match rust(r) {
        Ok(left) => left,
        Err(e) => {
            restore(regions, &before);
            unsafe { crate::fatal::raise::refused(hook, e, r, call_original) };
            return;
        }
    };
    let mine = capture(regions);
    restore(regions, &before);
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let theirs = capture(regions);
    let agreement =
        compare_memory(regions, &theirs, &mine, floats).and(compare_regs(&left, r, &o, floats));
    match agreement {
        Agreement::Same => {}
        Agreement::Drift(n) => note_drift(hook.name(), || {
            format!("{n} places; memory: {}", diff_summary(&theirs, &mine))
        }),
        Agreement::Differ => crate::screens::harness::mismatch_unless_disturbed(hook, || {
            format!(
                "regs_ok={}; control word {word:#06x}; memory: {}",
                compare_regs(&left, r, &o, floats) != Agreement::Differ,
                diff_summary(&theirs, &mine)
            )
        }),
    }
    let esp = r.esp;
    *r = o;
    r.esp = esp;
}

/// `dispatch_state_floats` outside shadow mode, without its hook (a refusal is the
/// caller's).
pub fn floats_entry(r: &mut Regs, rust: Rust) -> Result<(), Refused> {
    rust(r)?.apply(r);
    Ok(())
}

/// `service::harness::check_state` with float places: a drift passes, and is noted.
#[allow(clippy::too_many_arguments)]
pub fn check_state_floats(
    t: &mut SelfTest,
    name: &'static str,
    what: &str,
    call_original: CallOriginal,
    before: Regs,
    prepare: &dyn Fn(),
    regions: &[Region],
    floats: &Floats,
    rust: Rust,
) {
    prepare();
    let mut ro = before;
    unsafe { call_original(&mut ro) };
    let theirs = capture(regions);
    prepare();
    let mine: Result<Left, Refused> = rust(&before);
    let state = capture(regions);
    let agreement = match &mine {
        Ok(left) => compare_memory(regions, &theirs, &state, floats)
            .and(compare_regs(left, &before, &ro, floats)),
        Err(_) => Agreement::Differ,
    };
    if let Agreement::Drift(n) = agreement {
        note_drift(name, || format!("self-test {what}: {n} places"));
    }
    t.check(agreement != Agreement::Differ, || {
        format!(
            "{what}: original eax={:#x} ebx={:#x} ecx={:#x} edx={:#x} esi={:#x} edi={:#x}; \
             rust {mine:x?}; memory: {}",
            ro.eax,
            ro.ebx,
            ro.ecx,
            ro.edx,
            ro.esi,
            ro.edi,
            diff_summary(&theirs, &state)
        )
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tolerances() {
        let one = 1.0f32.to_bits();
        assert!(close(one, one + 8, FLOAT));
        assert!(!close(one, one + 9, FLOAT));
        assert!(close(0x7FC0_0000, 0xFFC0_0000, Tolerance::Exact));
        assert!(close(0xABF8_68F7, 0xABF8_6900, SUM));
        assert!(!close(0xABF8_68F7, 0xABF8_6900, FLOAT));
        assert!(!close(one, one + 1, Tolerance::Exact));
        assert!(close(0x8000_0000, 0, FLOAT)); // minus and plus zero are neighbours
        assert!(close(3, 4, ROUNDED));
        assert!(!close(3, 5, ROUNDED));
    }
}
