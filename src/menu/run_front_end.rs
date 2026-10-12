//! `menu::run_front_end` (0x47486C): the game between its start and its end: the films
//! and the main menu, with everything that is reached from it.
//!
//! In:  nothing
//! Out: eax = 0; everything else preserved
//! `dword_755684` = `dword_4F82D8`; `menu::drop_missing_controllers`; `gfx::build_fonts`;
//! the films at the start (`sub_4161CD` with eax = 0, not ported); the log line "BEV:
//! Starting GB_Front"; `menu::main_menu_loop`; "BEV: Finished GB_Front";
//! `menu::store_player_names`; `gfx::free_fonts`; `saves::list_car_numbers`.
//! Callers: `system::main_loop` and 0x443260.
//! Ungated thunk: the whole game runs inside it. Shadow mode runs the original alone.
//! Each real call writes a line to hook.log saying which version runs.
//! Self-test: a redirected copy whose nine calls go to the test.
//! Refuses what its parts refuse; such a refusal comes with the work half done.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, log, rva,
    shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::run_front_end", rva(0x47486C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

// ---- game data ----

const FROM_RVA: usize = rva(0x4F82D8); // dword_4F82D8
const TO_RVA: usize = rva(0x755684); // dword_755684
const STARTING_RVA: usize = rva(0x4DE67B); // "BEV: Starting GB_Front"
const FINISHED_RVA: usize = rva(0x4DE692); // "BEV: Finished GB_Front"

// ---- Rust version ----

/// What the function calls, in the order of the test's stand-ins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Call {
    DropMissingControllers,
    BuildFonts,
    /// The films, with eax = 0.
    Films,
    /// A line of the game's log: the text's address.
    Log(u32),
    MainMenuLoop,
    StorePlayerNames,
    FreeFonts,
    ListCarNumbers,
}

pub type Outside<'a> = &'a mut dyn FnMut(Call) -> Result<(), Refused>;

pub unsafe fn run_front_end(outside: Outside) -> Result<(), Refused> {
    unsafe {
        game_write(TO_RVA, game_read::<u32>(FROM_RVA));
        outside(Call::DropMissingControllers)?;
        outside(Call::BuildFonts)?;
        outside(Call::Films)?;
        outside(Call::Log(game_addr(STARTING_RVA) as u32))?;
        outside(Call::MainMenuLoop)?;
        outside(Call::Log(game_addr(FINISHED_RVA) as u32))?;
        outside(Call::StorePlayerNames)?;
        outside(Call::FreeFonts)?;
        outside(Call::ListCarNumbers)
    }
}

// ---- hook ----

/// The calls done by the Rust versions; the films and the menu's loop through the game
/// (the loop's own hook decides which version of it runs).
fn in_game(call: Call) -> Result<(), Refused> {
    unsafe {
        match call {
            Call::DropMissingControllers => {
                crate::menu::drop_missing_controllers::drop_missing_controllers()?
            }
            Call::BuildFonts => call_game(&mut Regs::default(), game_addr(rva(0x415563))),
            Call::Films => call_game(&mut Regs::default(), game_addr(rva(0x4161CD))),
            Call::Log(text) => crate::menu::text_entry::log_l1(text as *const u8, true),
            Call::MainMenuLoop => call_game(&mut Regs::default(), game_addr(rva(0x4748D9))),
            Call::StorePlayerNames => {
                crate::menu::store_player_names::store_player_names(&mut |line| {
                    crate::menu::text_entry::log_l1(line, true)
                })?
            }
            Call::FreeFonts => crate::gfx::free_fonts::free_fonts(
                game_addr(crate::gfx::glyph_tables::TABLES_LIST_RVA)
                    as *mut crate::memory::MemoryList,
                game_addr(crate::gfx::build_fonts::PALETTES_RVA) as *mut u32,
            )?,
            Call::ListCarNumbers => crate::saves::list_car_numbers::list_car_numbers()?,
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        log("menu::run_front_end: the original runs");
        unsafe { call_original(r) };
        return;
    }
    log("menu::run_front_end: the Rust version runs");
    match unsafe { run_front_end(&mut in_game) } {
        Ok(()) => {
            r.eax = 0;
            log("menu::run_front_end: the Rust version has come back");
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test ----

const LEN: usize = 0x4A;
const ENTRY_BYTES: [u8; 8] = [0xA1, 0xD8, 0x82, 0x4F, 0x00, 0xA3, 0x84, 0x56];
const CALLS: [usize; 9] = [0xA, 0xF, 0x16, 0x22, 0x28, 0x34, 0x3A, 0x3F, 0x44];
/// The callees, in the order of the stand-ins' places.
const CALLEES: [usize; 8] = [
    0x4748B6, 0x415563, 0x4161CD, 0x40142B, 0x4748D9, 0x470649, 0x415C55, 0x470825,
];

/// Each call: its place and eax.
static CALLED: Mutex<Vec<(u32, u32)>> = Mutex::new(Vec::new());

fn handler(place: u32, r: &mut Regs) {
    // Only the films and the log lines are given something in eax.
    let eax = if place == 2 || place == 3 { r.eax } else { 0 };
    CALLED.lock().unwrap().push((place, eax));
}

fn through_the_test(call: Call) -> Result<(), Refused> {
    let (place, eax) = match call {
        Call::DropMissingControllers => (0, 0),
        Call::BuildFonts => (1, 0),
        Call::Films => (2, 0),
        Call::Log(text) => (3, text),
        Call::MainMenuLoop => (4, 0),
        Call::StorePlayerNames => (5, 0),
        Call::FreeFonts => (6, 0),
        Call::ListCarNumbers => (7, 0),
    };
    act(
        place,
        &mut Regs {
            eax,
            ..Regs::default()
        },
    );
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x47486C)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| {
                CALLEES
                    .iter()
                    .position(|&known| rva(known) == callee)
                    .map(|place| stand_in(place as u32))
            },
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions = [at(TO_RVA)];
    let inputs = [at(FROM_RVA)];
    let saved = (capture(&regions), capture(&inputs));
    for value in [0u32, 0x40, 0xDEAD_BEEF] {
        let before = Regs::SENTINEL;
        let run = |rust: bool| unsafe {
            game_write(TO_RVA, 0xAAAA_AAAAu32);
            game_write(FROM_RVA, value);
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let ended = if rust {
                run_front_end(&mut through_the_test).is_err() as u32
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                (ro.eax != 0 || !ro.preserved_except(&before, crate::framework::EAX)) as u32
            };
            (ended, capture(&regions), CALLED.lock().unwrap().clone())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 == 0, || {
            format!("value {value:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
