//! `physics::checkpoint_offset` (0x4415E7): the work car's place against its checkpoint
//! (`g_car` +0x98). With a checkpoint after it (+0x98 + 1 below `g_checkpointCount`, by
//! the sign of the difference): the checkpoint's face (`g_checkpointFaces`), that face's
//! first point (`g_courseFaces` +4, `g_scenePoints`), the car's x and z less the point's,
//! turned by minus the checkpoint's angle (`dword_9ED808`, `math::turn_point_xz`) into
//! +0x620 and +0x624. Else both 0.
//!
//! In:  nothing
//! Out: eax: x' (or +0x98 + 1); with a checkpoint after it also ebx the point's number,
//!      ecx z', edx as `turn_point_xz` leaves it, esi the point; the rest kept; no flags
//!      read (the caller returns)
//!
//! Caller: `sub_44157B`.
//! Refuses (before anything) a checkpoint below 0 or past 0x1418, or no faces or points.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back) with test faces and points: checkpoints at, before and past the last.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    EAX, EBX, ECX, EDX, ESI, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm,
    rva,
};
use crate::math::turn_point_xz::turn_point_xz;
use crate::math::turn_scratch_xz::scratch;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::checkpoint_offset",
    rva(0x4415E7),
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

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

const CHECKPOINT: usize = 0x71A070; // g_car + 0x98
const COUNT: usize = 0x623640; // g_checkpointCount
const ANGLES: usize = 0x9ED808; // dword_9ED808
const CHECK_FACES: usize = 0x9E3738; // g_checkpointFaces
const FACES: usize = 0x5CCCBC; // g_courseFaces
const POINTS: usize = 0x5CCCC0; // g_scenePoints
const OFFSET: usize = 0x71A5F8; // g_car + 0x620

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust version.
pub unsafe fn checkpoint_offset() -> Result<Left, Refused> {
    let c = g(CHECKPOINT);
    let next = c.wrapping_add(1);
    if (next.wrapping_sub(g(COUNT)) as i32) >= 0 {
        p(OFFSET, 0);
        p(OFFSET + 4, 0);
        return Ok(Left::nothing().with(EAX, next));
    }
    if c > 0x1418 {
        return Err(BadInput("a checkpoint below 0 or past 0x1418").into());
    }
    if g(FACES) == 0 || g(POINTS) == 0 {
        return Err(BadInput("no course faces or points").into());
    }
    unsafe {
        let angle = g(ANGLES + 4 * c as usize);
        let face = g(CHECK_FACES + 4 * c as usize);
        let f = (face << 6).wrapping_add(g(FACES));
        let point = at(f + 4);
        let pt = point.wrapping_mul(12).wrapping_add(g(POINTS));
        let x = g(0x71A10C).wrapping_sub(at(pt));
        let z = g(0x71A114).wrapping_sub(at(pt + 8));
        let [x2, _, z2, entry] =
            turn_point_xz(x as i32, point as i32, z as i32, angle.wrapping_neg());
        p(OFFSET, x2);
        p(OFFSET + 4, z2);
        Ok(Left::nothing()
            .with(EAX, x2)
            .with(EBX, point)
            .with(ECX, z2)
            .with(EDX, entry)
            .with(ESI, pt))
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { checkpoint_offset() }
}

fn regions() -> Vec<Region> {
    vec![(game_addr(rva(OFFSET)) as u32, 8), scratch()]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // Two faces of 0x40 bytes, four points of 12.
    let mut faces = vec![0u32; 32];
    let mut points = vec![0u32; 12];
    let (fa, pa) = (faces.as_mut_ptr() as u32, points.as_mut_ptr() as u32);
    let a = |x: usize| game_addr(rva(x)) as u32;
    let mut all = regions();
    all.extend([
        (a(CHECKPOINT), 4),
        (a(COUNT), 4),
        (a(ANGLES), 16),
        (a(CHECK_FACES), 16),
        (a(FACES), 8),
        (a(0x71A10C), 12),
    ]);
    let saved = capture(&all);
    for (n, (c, count)) in [(0u32, 5u32), (1, 5), (3, 5), (4, 5), (2, 3), (0, 0)]
        .into_iter()
        .enumerate()
    {
        let n = n as u32;
        let set = || unsafe {
            for k in 0..32u32 {
                ((fa + 4 * k) as *mut u32).write(0x7700_0000 | k);
            }
            ((fa + 4) as *mut u32).write(1);
            ((fa + 0x44) as *mut u32).write(3);
            for k in 0..12u32 {
                ((pa + 4 * k) as *mut u32).write(k.wrapping_mul(0x1_2345) ^ (n << 20));
            }
            p(FACES, fa);
            p(POINTS, pa);
            p(CHECKPOINT, c);
            p(COUNT, count);
            for k in 0..4 {
                p(ANGLES + 4 * k, (k as u32 + n).wrapping_mul(0x3456_789B));
                p(CHECK_FACES + 4 * k, (k as u32 + n) % 2);
            }
            p(0x71A10C, 0x0123_4567u32.wrapping_mul(n + 1));
            p(0x71A114, 0x0765_4321u32.wrapping_mul(n + 3));
        };
        check_state(
            &mut t,
            &format!("checkpoint {c}, count {count}"),
            call_original,
            Regs::SENTINEL,
            &set,
            &all,
            &rust,
        );
    }
    restore(&all, &saved);
    drop((faces, points));
    t.finish()
}
