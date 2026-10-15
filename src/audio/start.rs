//! `audio::start` (0x40A01D): the sound device probed at the audio thread's start, and the
//! two sound modes chosen.
//!
//! In:  nothing
//! Out: carry = no race mode (no sound); every register preserved
//!
//! `audio::open_a3d2` (eax 0), `g_hasA3d2` = `g_useA3d`, `audio::close_a3d2`; the mode
//! 0x11221 | 0x1900000; `audio::open_sound` (eax 0). Opened: `g_a3dSplashKept` =
//! `g_a3dReady`; `audio::read_caps`; unless it fails or the caps say emulated (0x20):
//! `audio::make_primary`, `g_has3d` from the mode's 3D bit, `audio::open_way`, a probe voice
//! in `g_voices[0]` (zeroed; format 0x121, `g_mixRateSetting`, flags 1, 0x4000 bytes)
//! through `audio::make_buffer` (ebx 0x11221, esi the voice), `release_eax`,
//! `release_voice_manager`, `release_voice`, `audio::close_way`, `audio::release_primary`; with
//! caps flag 0x100, free hardware buffers and 3D: race bits 0x112221 for A3D2, else with
//! `g_voiceManagerOn` and at least 6 free 3D buffers 0x900000 (and 0x1000000 with `g_hasEax`). Not
//! opened: 0x112221 with A3D2, else both modes 0 and straight to the log. Then: unless
//! both modes are kept, the race mode is the probe's and at once the menu's 0x11221 (the
//! probe thrown away: section 16); `audio::choose_modes`; "AUD: Using modes" and both;
//! `audio::close_sound`; the mode 0. One caller, the audio thread.
//! Shadow mode runs the original alone. Self-test: a redirected copy whose calls go to
//! stand-ins noting what each reads, 20 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::voices::{read, write};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::install::stand_ins::{set_handler, stand_in};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::start", rva(0x40A01D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The calls by place (offsets in `CALLS`); `system::zero_memory` (0x8C) is done inline.
const CALLEES: [usize; 17] = [
    0x40CA17, // 0 open_a3d2 (eax)
    0x40CAD9, // 1 close_a3d2
    0x409E1D, // 2 open_sound (eax)
    0x409EF8, // 3 read_caps
    0x40A54B, // 4 make_primary
    0x40AC3A, // 5 open_way
    0x40B5CF, // 6 make_buffer (ebx, esi)
    0x40D2F0, // 7 release_eax
    0x40C74C, // 8 release_voice_manager
    0x40B767, // 9 release_voice (esi)
    0x40AC58, // 10 close_way
    0x40A61F, // 11 release_primary
    0x40A1CA, // 12 choose_modes
    0x401432, // 13 log_write_l2 (eax)
    0x40162D, // 14 log_hex8_l2 (eax)
    0x401634, // 15 log_hex8_l2_nl (eax)
    0x409EA5, // 16 close_sound
];
const CALLS: [usize; 18] = [
    0x03, 0x12, 0x2E, 0x43, 0x5E, 0x78, 0x8C, 0xAF, 0xB4, 0xB9, 0xBE, 0xC3, 0xC8, 0x14D, 0x159,
    0x165, 0x171, 0x177,
];
const USING_MODES_TEXT: usize = rva(0x4DA6AD); // "AUD: Using modes"

#[derive(Clone, Copy)]
pub struct StartVars {
    pub use_a3d: u32,
    pub has_a3d2: u32,
    pub mode: u32,
    pub a3d_ready: u32,
    pub splash_kept: u32,
    /// `g_soundCaps`: flags at +4, free hardware buffers at +0x14, free 3D ones at +0x3C.
    pub caps: u32,
    pub has_3d: u32,
    pub voices: u32,
    pub mix_rate_setting: u32,
    pub voice_manager_on: u32,
    pub has_eax: u32,
    pub menu: u32,
    pub race: u32,
}

impl StartVars {
    pub fn game() -> Self {
        let a = |r: usize| game_addr(rva(r)) as u32;
        Self {
            use_a3d: a(0x506D60),
            has_a3d2: a(0x506D5C),
            mode: a(0x4F82BC),
            a3d_ready: a(0x506C64),
            splash_kept: a(0x506C60),
            caps: a(0x506AA8),
            has_3d: a(0x506B46),
            voices: a(0x4F834C),
            mix_rate_setting: a(0x4F82D4),
            voice_manager_on: a(0x506C54),
            has_eax: a(0x506C70),
            menu: a(0x4F82C4),
            race: a(0x4F82C8),
        }
    }
}

/// A call (its place, the registers the original has there) -> its carry.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<bool, Refused>;

/// False for the carry (no race mode).
pub unsafe fn start(entry: &Regs, v: &StartVars, part: Part) -> Result<bool, Refused> {
    unsafe {
        let mut c = *entry;
        let mut call = |place: u32, c: &Regs| -> Result<bool, Refused> {
            let mut r = *c;
            part(place, &mut r)
        };
        c.eax = 0;
        call(0, &c)?;
        c.eax = read(v.use_a3d);
        write(v.has_a3d2, c.eax);
        call(1, &c)?;
        c.ebx = 0x1_1221;
        write(v.mode, c.ebx | 0x190_0000);
        c.eax = 0;
        let mut to_log = false;
        if call(2, &c)? {
            if read(v.has_a3d2) == 1 {
                c.ebx = 0x11_2221;
            } else {
                c.eax = 0;
                write(v.menu, 0);
                write(v.race, 0);
                to_log = true;
            }
        } else {
            c.eax = read(v.a3d_ready);
            write(v.splash_kept, c.eax);
            let flags = |v: &StartVars| read(v.caps + 4);
            if !call(3, &c)? && flags(v) & 0x20 == 0 {
                call(4, &c)?;
                c.eax = (read(v.mode) & 0x10_0000 != 0) as u32;
                (v.has_3d as *mut u32).write_unaligned(c.eax);
                call(5, &c)?;
                c.esi = v.voices;
                std::ptr::write_bytes(c.esi as *mut u8, 0, 0x70);
                write(c.esi + 0x28, 0x121);
                c.eax = read(v.mix_rate_setting);
                write(c.esi + 0x60, c.eax);
                write(c.esi, 1);
                write(c.esi + 0x20, 0x4000);
                for place in 6..12 {
                    call(place, &c)?;
                }
                if flags(v) & 0x100 != 0
                    && read(v.caps + 0x14) != 0
                    && (v.has_3d as *const u32).read_unaligned() == 1
                {
                    if read(v.has_a3d2) == 1 {
                        c.ebx = 0x11_2221;
                    } else if read(v.voice_manager_on) == 1 && read(v.caps + 0x3C) >= 6 {
                        c.ebx |= 0x90_0000;
                        if read(v.has_eax) == 1 {
                            c.ebx |= 0x100_0000;
                        }
                    }
                }
            }
        }
        if !to_log {
            if read(v.race) == 0 || read(v.menu) == 0 {
                write(v.race, c.ebx);
                c.ebx = 0x1_1221;
                write(v.menu, c.ebx);
                write(v.race, c.ebx);
            }
            call(12, &c)?;
        }
        for (place, eax) in [
            (13, game_addr(USING_MODES_TEXT) as u32),
            (14, read(v.menu)),
            (15, read(v.race)),
        ] {
            call(place, &Regs { eax, ..c })?;
        }
        call(16, &c)?;
        write(v.mode, 0);
        Ok(read(v.race) >= 1)
    }
}

pub fn game_part(place: u32, r: &mut Regs) -> Result<bool, Refused> {
    use crate::audio::sound_device as sd;
    unsafe {
        Ok(match place {
            0 => !crate::audio::a3d::open_a3d2(
                &crate::audio::a3d::A3dVars::game(),
                &mut crate::audio::a3d::game_create,
            )?,
            1 => {
                crate::audio::a3d::close_a3d2(
                    game_addr(crate::audio::a3d::A3D2_INTERFACE_RVA) as u32,
                    game_addr(crate::audio::voices::A3D_RVA) as u32,
                    &mut sd::game_part,
                )?;
                false
            }
            2 => !sd::open_sound(
                r.eax,
                &sd::OpenVars::game(),
                &mut sd::game_part,
                &mut sd::game_direct_sound_create,
            )?,
            3 => !sd::read_caps(
                game_addr(crate::audio::make_buffer::DIRECT_SOUND_RVA) as u32,
                game_addr(sd::CAPS_RVA) as u32,
                &mut sd::game_part,
            )?,
            4 => !crate::audio::primary::make_primary(
                &crate::audio::primary::PrimaryVars::game(),
                &mut sd::game_part,
            )?,
            5 => crate::audio::mixer_set_up::open_way(
                crate::framework::game_read(crate::audio::voices::SOUND_MODE_RVA),
                &mut crate::audio::restore::game_part,
            )?,
            6 => !crate::audio::make_buffer::make_buffer(
                r.esi,
                r.ebx,
                &crate::audio::make_buffer::Vars::game(),
                &mut crate::audio::make_buffer::game_part,
            )?,
            7 => {
                crate::audio::release_eax::release_eax(crate::audio::release_eax::game_vars());
                false
            }
            8 => {
                crate::audio::release_voice_manager::release_voice_manager(game_addr(
                    crate::audio::voices::VOICE_MANAGER_RVA,
                )
                    as u32);
                false
            }
            9 => {
                crate::audio::release_voice::release_voice_in_game(r.esi);
                false
            }
            10 => crate::audio::mixer_set_up::close_way(
                crate::framework::game_read(crate::audio::voices::SOUND_MODE_RVA),
                &mut crate::audio::restore::game_part,
            )?,
            11 => {
                sd::release_primary(
                    game_addr(crate::audio::voices::PRIMARY_RVA) as u32,
                    game_addr(sd::VOLUME_KEPT_RVA) as u32,
                    &mut sd::game_part,
                )?;
                false
            }
            12 => {
                crate::audio::choose_modes::in_game(&mut crate::audio::choose_modes::real_log)?;
                false
            }
            13 => sd::game_part(0, r.eax)?,
            14 => sd::game_part(2, r.eax)?,
            15 => sd::game_part(3, r.eax)?,
            _ => {
                sd::close_sound(
                    game_addr(crate::audio::make_buffer::DIRECT_SOUND_RVA) as u32,
                    game_addr(sd::WINDOW_RVA) as u32,
                    &mut sd::game_part,
                )?;
                false
            }
        })
    }
}

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    let ok = unsafe { start(r, &StartVars::game(), &mut game_part) }?;
    Ok(Leaves {
        eax: r.eax,
        carry: Some(!ok),
        zero: None,
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_leaf(&HOOK, r, call_original, rust);
}

// ---- self-test ----

/// Each call: the place and the registers it reads (eax, ebx, esi; 0 where it reads none).
static NOTED: Mutex<Vec<[u32; 4]>> = Mutex::new(Vec::new());
/// The carries by place.
static CARRIES: Mutex<[bool; 17]> = Mutex::new([false; 17]);

fn test_part(place: u32, r: &mut Regs) -> Result<bool, Refused> {
    let eax = if matches!(place, 0 | 2 | 13 | 14 | 15) {
        r.eax
    } else {
        0
    };
    let ebx = if place == 6 { r.ebx } else { 0 };
    let esi = if matches!(place, 6 | 9) { r.esi } else { 0 };
    NOTED.lock().unwrap().push([place, eax, ebx, esi]);
    Ok(CARRIES.lock().unwrap()[place as usize])
}

fn handler(place: u32, r: &mut Regs) {
    let carry = test_part(place, r).unwrap_or(false);
    r.set_carry(carry);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // 0 use A3D, 1 has A3D2, 2 mode, 3 ready, 4 splash, 5 has 3D, 6 mix rate, 7 A3D RM,
    // 8 has EAX, 9 menu, 10 race; then the caps (0x18 dwords) and a voice (0x1C).
    let mut m = vec![0u32; 11 + 0x18 + 0x1C];
    let at = m.as_mut_ptr() as u32;
    let a = |n: u32| at + n * 4;
    let v = StartVars {
        use_a3d: a(0),
        has_a3d2: a(1),
        mode: a(2),
        a3d_ready: a(3),
        splash_kept: a(4),
        has_3d: a(5),
        mix_rate_setting: a(6),
        voice_manager_on: a(7),
        has_eax: a(8),
        menu: a(9),
        race: a(10),
        caps: a(11),
        voices: a(11 + 0x18),
    };
    let mut variables = vec![
        (0x506D60usize, v.use_a3d),
        (0x506D5C, v.has_a3d2),
        (0x4F82BC, v.mode),
        (0x506C64, v.a3d_ready),
        (0x506C60, v.splash_kept),
        (0x506B46, v.has_3d),
        (0x4F82D4, v.mix_rate_setting),
        (0x506C54, v.voice_manager_on),
        (0x506C70, v.has_eax),
        (0x4F82C4, v.menu),
        (0x4F82C8, v.race),
        (0x4F834C, v.voices),
    ];
    for off in [4u32, 0x14, 0x3C] {
        variables.push((0x506AA8 + off as usize, v.caps + off));
    }
    let c = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40A01D,
            len: 0x1AD,
            entry: &[0x60, 0x33, 0xC0, 0xE8, 0xF2, 0x29, 0x00, 0x00],
            calls: &CALLS,
            jumps_out: &[],
        },
        &|callee| {
            CALLEES
                .iter()
                .position(|&c| rva(c) == callee)
                .map(|n| stand_in(n as u32))
        },
        &variables,
        30,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    // (open fails, caps fail, caps flags, A3D2, EAX, kept modes)
    let mut cases = Vec::new();
    for open_fails in [false, true] {
        for has_a3d2 in [0u32, 1] {
            for kept in [false, true] {
                cases.push((open_fails, false, 0x100u32, has_a3d2, 0u32, kept));
            }
        }
    }
    for (caps_fail, flags) in [(true, 0x100u32), (false, 0x20), (false, 0), (false, 0x120)] {
        cases.push((false, caps_fail, flags, 0, 1, false));
    }
    for has_eax in [0u32, 1] {
        for kept in [false, true] {
            for flags in [0x100u32, 0x1100] {
                cases.push((false, false, flags, 0, has_eax, kept));
            }
        }
    }
    for (open_fails, caps_fail, flags, has_a3d2, has_eax, kept) in cases {
        let run = |rust: bool| {
            unsafe {
                let p = at as *mut u32;
                for n in 0..m.len() {
                    p.add(n).write_volatile(0x5555_5555);
                }
                for (n, x) in [
                    (0, 0),
                    (1, 0x77),
                    (2, 0x77),
                    (3, 0),
                    (4, 0x77),
                    (5, 0x77),
                    (6, 22050),
                    (7, 1),
                    (8, has_eax),
                    (9, if kept { 0x11221 } else { 0 }),
                    (10, if kept { 0x2441220 } else { 0 }),
                ] {
                    p.add(n).write_volatile(x);
                }
                let caps = p.add(11);
                caps.add(1).write_volatile(flags);
                caps.add(5).write_volatile(4);
                caps.add(15).write_volatile(8);
                // `use_a3d` stands for what `open_a3d2` leaves: has A3D2 follows it.
                p.write_volatile(has_a3d2);
            }
            {
                let mut c = CARRIES.lock().unwrap();
                *c = [false; 17];
                c[2] = open_fails;
                c[3] = caps_fail;
            }
            NOTED.lock().unwrap().clear();
            let before = Regs::SENTINEL;
            let mut ro = before;
            let carry = if rust {
                unsafe { start(&before, &v, &mut test_part) }
                    .ok()
                    .map(|ok| !ok)
            } else {
                unsafe { call_game(&mut ro, c.addr()) };
                ro.preserved_except(&before, 0).then(|| ro.carry())
            };
            (
                carry,
                std::mem::take(&mut *NOTED.lock().unwrap()),
                unsafe { std::slice::from_raw_parts(at as *const u32, m.len()) }.to_vec(),
            )
        };
        let theirs = run(false);
        let mine = run(true);
        t.check(theirs.0.is_some() && theirs == mine, || {
            format!(
                "open fails {open_fails}, caps fail {caps_fail}, flags {flags:#x}, A3D2 {has_a3d2}, EAX {has_eax}, kept {kept}: original {:?} {:x?}; rust {:?} {:x?}",
                theirs.0, theirs.1, mine.0, mine.1
            )
        });
    }
    *CARRIES.lock().unwrap() = [false; 17];
    drop(m);
    t.finish()
}
