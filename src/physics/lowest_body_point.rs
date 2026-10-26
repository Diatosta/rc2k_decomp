//! `physics::lowest_body_point` (0x45D6DB): with more than four probe points
//! (`dword_71A238`), the deepest of the body's (points 4 .., their depths
//! `dword_6D2514[n]`, from 0x640000 down by the sign of the difference) into
//! `dword_6D3068` and its number into `dword_6D306C` (-1 for none); then `flt_6D3070` =
//! `dword_6D3068` / 65536 (with four or fewer, of what it held).
//!
//! In:  nothing
//! Out: eax the last depth looked at, ebx the count (when there are more than four); the
//!      rest kept; no flags read (callers go on with `mov`, `cmp`)
//!
//! Callers: `physics::feel_ground`, `sub_43BDD1` (twice), `sub_45EDB5`, `sub_45EFF3`.
//! Refuses (before anything) a count above 0x40 (the depths' room).
//! Self-test: `physics/copy_hooks_test.rs` (`test_moves_floats`: a copy on test
//! variables against the converted version), counts 0 .. 0x40; the refusal.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EBX, RawHook, Regs, SelfTest, game_addr, game_read, hook_asm, rva};
use crate::math::x87;
use crate::physics::copy_hooks_test::{lot, test_moves_floats};
use crate::screens::harness::Left;
use crate::service::floats::{FLOAT, Floats, dispatch_state_floats, floats_entry};
use rc2k_core::physics::motion::{lowest_point, lowest_share};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::lowest_body_point",
    rva(0x45D6DB),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The count, the lowest, its number, the share, then the 0x40 depths.
fn variables() -> Vec<usize> {
    let mut v = vec![0x71A238, 0x6D3068, 0x6D306C, 0x6D3070];
    v.extend((0..0x40).map(|k| 0x6D2514 + 4 * k));
    v
}

/// The Rust core on the variables at `v`; `r` as the original leaves it.
pub unsafe fn lowest_at(v: &[u32], r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        let get = |k: usize| (v[k] as *const u32).read_unaligned();
        let put = |k: usize, x: u32| (v[k] as *mut u32).write_unaligned(x);
        let n = get(0);
        if n > 0x40 {
            return Err(BadInput("more than 0x40 probe points (the depths' room)").into());
        }
        if n > 4 {
            put(1, 0x64_0000);
            put(2, u32::MAX);
            r.ebx = 4;
            loop {
                r.eax = get(4 + r.ebx as usize);
                if (r.eax.wrapping_sub(get(1)) as i32) < 0 {
                    put(1, r.eax);
                    put(2, r.ebx);
                }
                r.ebx += 1;
                if r.ebx == n {
                    break;
                }
            }
        }
        put(3, x87::int_div(get(1) as i32, game_read(rva(0x5C5D98))));
        Ok(())
    }
}

/// The converted version (`rc2k_core::physics::motion`) on the variables at `v`; `r` as
/// the original leaves it.
// Converted: `rc2k_core::physics::motion::{lowest_point, lowest_share}`.
pub unsafe fn lowest_at_plain(v: &[u32], r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        let get = |k: usize| (v[k] as *const u32).read_unaligned();
        let put = |k: usize, x: u32| (v[k] as *mut u32).write_unaligned(x);
        let (mut lowest, mut number) = (get(1), get(2));
        if let Some((eax, ebx)) =
            lowest_point(get(0), |n| get(4 + n as usize), &mut lowest, &mut number)?
        {
            r.eax = eax;
            r.ebx = ebx;
            put(1, lowest);
            put(2, number);
        }
        let scale = f32::from_bits(game_read(rva(0x5C5D98)));
        put(3, lowest_share(lowest, scale).to_bits());
        Ok(())
    }
}

fn game() -> Vec<u32> {
    variables()
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version (the exact one, for `physics::feel_ground`, `ground_contact`,
/// `find_clear_place`, `crash_contact`); `r` as the original leaves it.
pub unsafe fn lowest_body_point(r: &mut Regs) -> Result<(), Refused> {
    unsafe { lowest_at(&game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { lowest_at_plain(&game(), &mut g) }?;
    Ok(Left::nothing().with(EAX, g.eax).with(EBX, g.ebx))
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    let floats = Floats {
        memory: &[((v[3], 4), FLOAT)],
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &[(v[1], 12)], &floats, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}

fn fill(seed: u32, out: &mut [u32]) {
    lot(seed, out);
    out[0] = [0u32, 4, 5, 6, 12, 0x40][(seed % 6) as usize];
    for o in out.iter_mut().skip(4) {
        *o = (*o % 0x100_0000).wrapping_sub(0x40_0000);
    }
}

fn self_test() -> (usize, usize) {
    let vars = variables();
    let mut result = test_moves_floats(
        &HOOK,
        0x45D6DB,
        0x5C,
        &[0x83, 0x3D, 0x38, 0xA2, 0x71, 0x00, 0x04],
        &vars,
        10,
        12,
        &fill,
        &|v, r| {
            let _ = unsafe { lowest_at_plain(v, r) };
            Left::nothing().with(EAX, r.eax).with(EBX, r.ebx)
        },
        &[(3, FLOAT)],
        &[],
    );
    if let Some(mut t) = SelfTest::begin(&HOOK) {
        let mut cells = vec![0u32; vars.len()];
        cells[0] = 0x41;
        let at: Vec<u32> = cells.iter().map(|c| c as *const u32 as u32).collect();
        let refused = unsafe { lowest_at_plain(&at, &mut Regs::default()) }.is_err();
        t.check(refused, || "a count of 0x41 not refused".into());
        let (c, f) = t.finish();
        result.0 += c;
        result.1 += f;
    }
    result
}
