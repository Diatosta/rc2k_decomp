//! `marks::init_marks` (0x4D12F7): the marks set up for a stage: the settings and the
//! seed kept, then the fifteen steps (each after `nullsub_1` with its number in eax), the
//! screen's middle and the kind's float.
//!
//! In:  esi the settings, eax the seed
//! Out: eax the kind's float (bits), ebp the kind, esi the settings; the rest as
//!      `marks::tick_clock` leaves them
//!
//! One caller, init mark's end (0x457030). Shadow mode runs the original alone (its steps
//! take memory and read files). Refuses (section 15) no settings, and a kind (the
//! settings' +0x26A8, which step 1 makes `g_marksKind`) past `dword_765240`'s 0x17
//! entries; both before anything. Self-test: a redirected copy whose 30 calls go to
//! stand-ins noting every register, against the Rust version on the same stand-ins
//! (settings of the test's; three kinds and two screen sizes); put back afterwards.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::marks::masks::{NOTED, note_step, test_step};
use crate::math::x87;
use crate::race::level_init::Part;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::init_marks", rva(0x4D12F7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const KIND_AT: u32 = 0x26A8; // the settings' kind
const LIVE_AT: u32 = 0x18; // the settings' dword set to 1
const CLOCK_STARTED_RVA: usize = rva(0x764FC8); // g_marksClockStarted
const ONES_RVA: [usize; 2] = [rva(0x764FF0), rva(0x764FF4)]; // dword_764FF0, dword_764FF4
const CLEARED_RVA: usize = rva(0x76513C); // dword_76513C
const SCREEN_WIDTH_RVA: usize = rva(0x4F494C); // g_screenWidth
const SCREEN_HEIGHT_RVA: usize = rva(0x4F4950); // g_screenHeight
/// flt_7687E4 .. flt_7687F0: the screen's middle x, it less `flt_76516C`, the same for y.
const MIDDLE_RVA: usize = rva(0x7687E4);
const LESS_RVA: usize = rva(0x76516C); // flt_76516C
const KIND_FLOATS_RVA: usize = rva(0x765240); // dword_765240: 0x17 floats by kind
const KINDS: u32 = 0x17;
const KIND_FLOAT_RVA: usize = rva(0x76529C); // flt_76529C
/// The place of `nullsub_1` (before each step, eax the step's number).
const NUMBERED: u32 = 16;
const STEPS: u32 = 15;

/// The Rust version (see the header). Places 1 .. 15 the steps (7 `nullsub_26`), 16
/// `nullsub_1`.
pub unsafe fn init_marks(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    use crate::marks::lines::SETTINGS_RVA;
    let mut g = *entry;
    unsafe {
        if g.esi == 0 {
            return Err(BadInput("no marks settings").into());
        }
        if ((g.esi + KIND_AT) as *const u32).read_unaligned() >= KINDS {
            return Err(BadInput("a marks kind past dword_765240's entries").into());
        }
        game_write(SETTINGS_RVA, g.esi);
        game_write(crate::marks::setup::SEED_RVA, g.eax);
        game_write(CLOCK_STARTED_RVA, 0u32);
        for step in 1..STEPS {
            g.eax = step;
            part(NUMBERED, &mut g)?;
            part(step, &mut g)?;
        }
        g.eax = STEPS;
        part(NUMBERED, &mut g)?;
        g.esi = game_read(SETTINGS_RVA);
        ((g.esi + LIVE_AT) as *mut u32).write_unaligned(1);
        for r in ONES_RVA {
            game_write(r, 1u32);
        }
        part(STEPS, &mut g)?;
        game_write(CLEARED_RVA, 0u32);
        let less: u32 = game_read(LESS_RVA);
        for (n, side) in [SCREEN_WIDTH_RVA, SCREEN_HEIGHT_RVA]
            .into_iter()
            .enumerate()
        {
            g.eax = game_read::<u32>(side) >> 1;
            let to = MIDDLE_RVA + 8 * n;
            game_write(to, x87::from_int(g.eax as i32));
            game_write(to + 4, x87::int_less(g.eax as i32, less));
        }
        g.ebp = game_read(crate::marks::kind_values::KIND_RVA);
        g.eax = game_read(KIND_FLOATS_RVA + 4 * g.ebp as usize);
        game_write(KIND_FLOAT_RVA, g.eax);
    }
    Ok(g)
}

/// In the game: the steps' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::marks::*;
    unsafe {
        let left = match place {
            1 => edges::load_kind_data(r, &mut edges::kind_part)?,
            2 => setup::make_wave_tables(r)?,
            3 => profiles::make_profiles(
                r,
                game_addr(profiles::RESUME_RVA) as u32,
                &mut blocks::game_part,
            )?,
            4 => setup::seed_noise(r),
            5 => setup::make_noise_curve(r)?,
            6 => blocks::allocate_places(r, &mut blocks::game_part)?,
            8 => kind_values::set_kind_values(r, &kind_values::FIRST)?,
            9 => kind_values::set_kind_values(r, &kind_values::SECOND)?,
            10 => load_mask_pictures::load_mask_pictures(r, &mut load_mask_pictures::game_part)?,
            11 => setup::clear_slots(r),
            12 => set_up_surface::set_up_surface(r, &mut set_up_surface::game_part)?,
            13 => blocks::allocate_ring(r, &mut blocks::game_part)?,
            14 => lines::set_up_blocks(r, &mut lines::game_part)?,
            15 => setup::tick_clock(r),
            // `nullsub_26`, `nullsub_1`.
            _ => return Ok(()),
        };
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..left
        };
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { init_marks(r, &mut game_part) } {
        Ok(left) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..left
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::marks::kind_values::KIND_RVA;
    use crate::marks::lines::SETTINGS_RVA;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov g_marksSettings, esi
    let entry: &[u8] = &[0x89, 0x35, 0x28, 0x7A, 0x76, 0x00];
    let calls: [usize; 30] = [
        0x1A, 0x1F, 0x29, 0x2E, 0x38, 0x3D, 0x47, 0x4C, 0x56, 0x5B, 0x65, 0x6A, 0x74, 0x79, 0x83,
        0x88, 0x92, 0x97, 0xA1, 0xA6, 0xB0, 0xB5, 0xBF, 0xC4, 0xCE, 0xD3, 0xDD, 0xE2, 0xEC, 0x112,
    ];
    let copy = match crate::race::mud::copy(
        0x4D12F7,
        0x176,
        entry,
        &calls,
        &[
            (0x4C1235, 1),
            (0x4C4E2F, 2),
            (0x4C4E1C, 3),
            (0x4C1697, 4),
            (0x4C2247, 5),
            (0x4C38B1, 6),
            (0x4C65BF, 7),
            (0x4C7322, 8),
            (0x4C7FA4, 9),
            (0x4C8BD8, 10),
            (0x4C967B, 11),
            (0x4CB0BC, 12),
            (0x4CFAC5, 13),
            (0x4D05BD, 14),
            (0x4D2BDC, 15),
            (0x429B18, NUMBERED),
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::install::stand_ins::set_handler(note_step);
    let scene = crate::marks::lines::TestScene::new();
    let settings = scene.settings.as_ptr() as u32;
    let regions: Vec<Region> = [
        SETTINGS_RVA,
        crate::marks::setup::SEED_RVA,
        CLOCK_STARTED_RVA,
        ONES_RVA[0],
        ONES_RVA[1],
        CLEARED_RVA,
        SCREEN_WIDTH_RVA,
        SCREEN_HEIGHT_RVA,
        KIND_FLOAT_RVA,
        KIND_RVA,
    ]
    .map(at)
    .into_iter()
    .chain([(game_addr(MIDDLE_RVA) as u32, 16)])
    .collect();
    let saved = capture(&regions);
    let live = (settings + LIVE_AT, 4);
    let mut watched = regions.clone();
    watched.push(live);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for (kind, screen) in [
        (0u32, None),
        (7, Some((641u32, 479u32))),
        (0x16, Some((1, 0))),
    ] {
        let prepare = || unsafe {
            restore(&regions, &saved);
            ((settings + KIND_AT) as *mut u32).write_unaligned(kind);
            ((settings + LIVE_AT) as *mut u32).write_unaligned(0xAAAA_AAAA);
            // Step 1 (a stand-in here) makes the kind `g_marksKind`.
            game_write(KIND_RVA, kind);
            if let Some((w, h)) = screen {
                game_write(SCREEN_WIDTH_RVA, w);
                game_write(SCREEN_HEIGHT_RVA, h);
            }
            NOTED.lock().unwrap().clear();
        };
        let start = Regs {
            esi: settings,
            eax: 0x1357_9BDF,
            ..Regs::SENTINEL
        };
        prepare();
        let mut theirs = start;
        unsafe { crate::framework::call_game(&mut theirs, copy.addr()) };
        let theirs = (
            regs(&theirs),
            std::mem::take(&mut *NOTED.lock().unwrap()),
            capture(&watched),
        );
        prepare();
        let mine = unsafe { init_marks(&start, &mut test_step) };
        let mine = (
            mine.as_ref().map(regs).unwrap_or_default(),
            std::mem::take(&mut *NOTED.lock().unwrap()),
            capture(&watched),
        );
        t.check(theirs == mine, || {
            format!("kind {kind:#x}, screen {screen:?}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    // The refusals: no settings, a kind past the table; nothing done.
    for (esi, kind) in [(0, 0), (settings, KINDS)] {
        unsafe {
            restore(&regions, &saved);
            ((settings + KIND_AT) as *mut u32).write_unaligned(kind);
            NOTED.lock().unwrap().clear();
        }
        let before = capture(&regions);
        let got = unsafe {
            init_marks(
                &Regs {
                    esi,
                    ..Regs::SENTINEL
                },
                &mut test_step,
            )
        };
        let calls = NOTED.lock().unwrap().len();
        t.check(
            got.is_err() && calls == 0 && capture(&regions) == before,
            || {
                format!(
                    "esi {esi:#x}, kind {kind:#x}: refused {}, {calls} calls",
                    got.is_err()
                )
            },
        );
    }
    restore(&regions, &saved);
    drop(scene);
    t.finish()
}
