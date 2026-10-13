//! `scene::reload_system_textures` (0x4254B4): every system texture gets the picture
//! it should have ("reload system plates").
//!
//! In:  nothing
//! Out: the registers as `scene::load_number_plates` leaves them
//!
//! - Two lines in the game's log (a rule and "Reload system plates - start").
//! - Each kind's first texture (`g_kindFirstTextures`), where it has one, wants the
//!   kind's picture (`dword_5D7968`; 0 counts as none, -1).
//! - For each of the `dword_5DC974` system textures: `dword_4F4A24` = 1 and the size
//!   pictures are read at (`dword_4F4A2C`, `dword_4F4A28`) = the scene's
//!   (`dword_609300`, `dword_609304`), or 256 x 256 for picture 0x3DE and pictures
//!   0x706 .. 0x7CF; `scene::load_changed_texture_picture`;
//!   `scene::step_loading_bar`.
//! - `dword_4F4A24` = 0 and the size 256 x 256 again; `scene::load_number_plates`
//!   (0x4255AA, not ported); two lines in the log.
//!
//! Two callers.
//! Refuses no system textures (the original goes through 2^32) or more than 0x400,
//! and a first texture past the table.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose calls note what was set by then.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_ptr, game_read, game_write, hook_asm, rva,
    shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::logfile::log_write::log_write;
use crate::redirect::RedirectedCopy;
use crate::scene::load_changed_texture_picture::{
    TEXTURES, WANTED_PICTURES_RVA, load_changed_texture_picture_in_game,
};
use crate::scene::step_loading_bar::step_loading_bar;
use crate::scene::textures::{KIND_TEXTURE_RVA, KINDS, test_table, write_table};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::reload_system_textures",
    rva(0x4254B4),
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

// ---- game data ----

const KIND_PICTURES_RVA: usize = rva(0x5D7968); // dword_5D7968
const SYSTEM_TEXTURES_RVA: usize = rva(0x5DC974); // dword_5DC974
const READING_RVA: usize = rva(0x4F4A24); // dword_4F4A24
/// `dword_4F4A28`, `dword_4F4A2C`: the size pictures are read at.
const READ_SIZE_RVA: usize = rva(0x4F4A28);
/// `dword_609300`, `dword_609304`: the scene's size for them.
const SCENE_SIZE_RVA: usize = rva(0x609300);
const PLATES_RVA: usize = rva(0x4255AA); // scene::load_number_plates
const TEXTS: [usize; 4] = [rva(0x4DB7E2), rva(0x4DB800), rva(0x4DB81D), rva(0x4DB83A)];

/// What the function calls.
pub enum Call {
    /// One of its four log lines.
    Log(usize),
    Picture(u32),
    Bar(u32),
    Plates,
}

// ---- Rust version ----

pub unsafe fn reload_system_textures(
    r: &mut Regs,
    call: &mut dyn FnMut(Call, &mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        let textures: u32 = game_read(SYSTEM_TEXTURES_RVA);
        if textures == 0 || textures > TEXTURES {
            return Err(BadInput("no system textures, or more than the table has").into());
        }
        let first = |kind: u32| game_read::<u32>(KIND_TEXTURE_RVA + kind as usize * 4);
        if (0..KINDS).any(|kind| first(kind) != u32::MAX && first(kind) >= TEXTURES) {
            return Err(BadInput("a kind's first texture past the pictures' table").into());
        }
        call(Call::Log(0), r)?;
        call(Call::Log(1), r)?;
        for kind in 0..KINDS {
            r.ebx = first(kind);
            if r.ebx == u32::MAX {
                continue;
            }
            r.eax = game_read(KIND_PICTURES_RVA + kind as usize * 4);
            if r.eax == 0 {
                r.eax = u32::MAX;
            }
            game_write(WANTED_PICTURES_RVA + r.ebx as usize * 4, r.eax);
        }
        let [width, height]: [u32; 2] = game_read(SCENE_SIZE_RVA);
        for texture in 0..textures {
            r.ecx = texture;
            game_write(READING_RVA, 1u32);
            r.eax = game_read(WANTED_PICTURES_RVA + texture as usize * 4);
            let own_size = r.eax == 0x3DE || (0x706..=0x7CF).contains(&r.eax);
            game_write(
                READ_SIZE_RVA,
                if own_size {
                    [0x100, 0x100]
                } else {
                    [height, width]
                },
            );
            call(Call::Picture(texture), r)?;
            call(Call::Bar(texture), r)?;
        }
        r.ecx = textures;
        game_write(READING_RVA, 0u32);
        game_write(READ_SIZE_RVA, [0x100u32, 0x100]);
        call(Call::Plates, r)?;
        call(Call::Log(2), r)?;
        call(Call::Log(3), r)?;
        Ok(())
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let entry = *r;
    let done = unsafe {
        reload_system_textures(r, &mut |call, r| {
            match call {
                Call::Log(n) => log_write(game_ptr(TEXTS[n]), true, 1),
                Call::Picture(texture) => load_changed_texture_picture_in_game(texture)?,
                Call::Bar(done) => {
                    let regs = Regs { eax: done, ..*r };
                    step_loading_bar(&regs, &mut |inner| {
                        let show = inner.esi as usize;
                        call_game(inner, show);
                    })?;
                }
                Call::Plates => call_game(r, game_addr(PLATES_RVA)),
            }
            Ok(())
        })
    };
    if let Err(e) = done {
        *r = entry;
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const START: usize = 0x4254B4;
const LEN: usize = 0xF6;
const ENTRY_BYTES: [u8; 7] = [0x60, 0x8D, 0x05, 0xE2, 0xB7, 0x4D, 0x00];
const CALL_SITES: [usize; 7] = [
    0x4254BB, 0x4254C8, 0x425554, 0x42555D, 0x42558A, 0x425596, 0x4255A3,
];

/// Each call: its place (0 a log line, 1 a picture, 2 the bar, 3 the plates), what it
/// was given (ecx for a picture, eax for the bar), and the reading switch and size
/// by then.
static CALLED: Mutex<Vec<[u32; 5]>> = Mutex::new(Vec::new());

fn handler(place: u32, r: &mut Regs) {
    unsafe {
        let [height, width]: [u32; 2] = game_read(READ_SIZE_RVA);
        let given = match place {
            1 => r.ecx,
            2 => r.eax,
            _ => 0,
        };
        CALLED
            .lock()
            .unwrap()
            .push([place, given, game_read(READING_RVA), height, width]);
    }
    if place == 3 {
        r.ebx = 0x55;
        r.esi = 0x66;
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let calls: Vec<usize> = CALL_SITES.iter().map(|site| site - START).collect();
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(START)),
            LEN,
            &ENTRY_BYTES,
            &calls,
            &|callee| {
                // logfile::log_write_l1_nl, the picture, the bar, the plates
                [rva(0x40142B), rva(0x42574B), rva(0x446786), PLATES_RVA]
                    .iter()
                    .position(|&known| known == callee)
                    .map(|place| stand_in(place as u32))
            },
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions: [Region; 3] = [
        (game_addr(WANTED_PICTURES_RVA) as u32, TEXTURES as usize * 4),
        at(READING_RVA),
        (game_addr(READ_SIZE_RVA) as u32, 8),
    ];
    let inputs: [Region; 4] = [
        (game_addr(KIND_TEXTURE_RVA) as u32, KINDS as usize * 4),
        (game_addr(KIND_PICTURES_RVA) as u32, KINDS as usize * 4),
        at(SYSTEM_TEXTURES_RVA),
        (game_addr(SCENE_SIZE_RVA) as u32, 8),
    ];
    let saved = (capture(&regions), capture(&inputs));
    let mut seed = 0x7E87_000Au32;
    for case in 0..10u32 {
        let firsts = test_table(&mut seed, KINDS, 2 + case % 3, 0x60);
        // Pictures of every range, 0 and none among them.
        let pictures: Vec<u32> = test_table(&mut seed, KINDS, 1, 0x7E0)
            .iter()
            .enumerate()
            .map(|(n, &picture)| match n % 7 {
                0 => 0,
                1 => 0x3DE,
                2 => 0x700 + picture % 0xD4,
                _ => picture,
            })
            .collect();
        let textures = [1u32, 2, 0x60, 0x33][case as usize % 4];
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.0.len()]);
            // What the textures want before the kinds say: left from a load before.
            write_table(WANTED_PICTURES_RVA, &pictures[..0x60]);
            write_table(KIND_TEXTURE_RVA, &firsts);
            write_table(KIND_PICTURES_RVA, &pictures);
            game_write(SYSTEM_TEXTURES_RVA, textures);
            game_write(SCENE_SIZE_RVA, [0x80u32 << (case % 2), 0x40]);
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let mut r = Regs::SENTINEL;
            if rust {
                let done = reload_system_textures(&mut r, &mut |call, r| {
                    match call {
                        Call::Log(_) => act(0, &mut { *r }),
                        Call::Picture(_) => act(1, &mut { *r }),
                        Call::Bar(done) => act(2, &mut Regs { eax: done, ..*r }),
                        Call::Plates => act(3, r),
                    }
                    Ok(())
                });
                if done.is_err() {
                    r = Regs::default();
                }
            } else {
                call_game(&mut r, copy.addr());
            }
            (
                [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp],
                capture(&regions),
                CALLED.lock().unwrap().clone(),
            )
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine, || {
            let first = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
            format!(
                "case {case}: {textures:#x} textures: regs {:x?} / {:x?}; {} / {} calls, the first that differs {first:?} ({:x?} / {:x?}); memory same {}",
                theirs.0,
                mine.0,
                theirs.2.len(),
                mine.2.len(),
                first.map(|n| theirs.2[n]),
                first.map(|n| mine.2[n]),
                theirs.1 == mine.1
            )
        });
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
