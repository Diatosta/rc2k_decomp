//! `saves::load_savegame` (0x46E6E1): the action of a slot on the "load game" menu: reads the
//! saved game, puts its state back into the game and goes on with the event from there.
//!
//! In:  nothing
//! Out: every register preserved
//! In order:
//!   - `sub_471F7E`; `dword_75592C` = 1, `dword_755250` = 0, `dword_755978` = 0;
//!   - `read_savegame_slot` on slot `dword_75523C * 10 + item - 1`. If that fails:
//!     `sub_40F059` with eax = 8, `menu::go_back`, and that is all;
//!   - the seven dwords, the four cars and the 180 records go back where `gather_savegame`
//!     took them from; `dword_75516C` is left pointing at the saved cars; between the dwords
//!     and the cars it calls `maybeCopySomeData` (only if `dword_755144` is 0) and
//!     `sub_4748B6`. Each car's 0x20 bytes at +0x124 are copied to where the car's pointer at
//!     +0x18 pointed before the car was overwritten; car 0's dwords at +0x14 and +0x274 go to
//!     `dword_7554C0` and `dword_7554C4`;
//!   - `sub_470D08` says where the event stands: 4 or 5, the next section starts (section + 1,
//!     stage index 0); 0 or 1, the next stage (index + 1); 3, nothing more happens here; any
//!     other value is fatal error 0x80702821 "Corrupt SaveGame";
//!   - unless it was 3: `dword_755970` = 1, `sub_47059E`, and then the event itself runs:
//!     `sub_4AA4D7` if `dword_7556D0` is 0x13, else `sub_4A0E60`. These only come back when
//!     the event is over or left;
//!   - `menu::go_back`.
//!
//! The eight functions named `sub_...` here are not ported and are called with `call_game`,
//! with the registers the original has at each call. Ten menu tables point at this function.
//! Refuses, before anything is done: a slot past 99, a missing player or a car without
//! its pointer at +0x18.
//! It runs the game, so shadow mode runs the original alone, and the thunk is not gated: a
//! whole event runs inside this call, and a gated thunk would count the game's thread as
//! "inside a hook" for all that time. The self-test runs a copy of the original whose eight
//! calls go to stand-ins that note the call; it loads a real saved game into the game's
//! state and puts everything back.
//! Manual patch with a private copy (0x1B8 bytes, 11 calls and one jump out relocated): a
//! call sits in the first bytes.

use crate::errors::{BadInput, FatalError, Refused};
use crate::files::numbered_path::buffer as path_buffer;
use crate::files::read_file::LAST_FILE_PATH_RVA;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::menu::go_back;
use crate::menu::name_lists::{NAME_LEN as SLOT_NAME_LEN, SAVEGAME_PAGE_RVA};
use crate::redirect::RedirectedCopy;
use crate::saves::gather_savegame::{
    CAR_LEN, CAR0_A_RVA, CAR0_B_RVA, CARS, HEAD_RVAS, NAME_LEN, NAME_OFFSET, NAME_PTR_OFFSET,
    RECORDS_LEN, RECORDS_RVA, cars,
};
use crate::saves::read_savegame_slot::read_savegame_slot;
use crate::saves::write_savegame_slot::{FILE_LEN, FILE_RVA, SLOTS, slot_name};
use crate::service::harness::{Region, at, capture, restore};
use crate::service::state::{MENU_ITEM_RVA, menu_item};
use std::ptr::copy_nonoverlapping;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("saves::load_savegame", rva(0x46E6E1), thunk, &ORIGINAL)
    .private_copy(0x1B8)
    .relocate_calls(&[1, 55, 157, 162, 311, 341, 389, 394, 411, 416, 428, 433])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

// ---- game data ----

const LOADING_RVA: usize = rva(0x75592C); // dword_75592C: set to 1
const CLEARED_A_RVA: usize = rva(0x755250); // dword_755250: set to 0
const CLEARED_B_RVA: usize = rva(0x755978); // dword_755978: set to 0
const SAVED_CARS_PTR_RVA: usize = rva(0x75516C); // dword_75516C: the saved cars in the buffer
const SKIP_COPY_RVA: usize = rva(0x755144); // dword_755144: 0 = `maybeCopySomeData` is called
const CONTINUING_RVA: usize = rva(0x755970); // dword_755970: set to 1 before the event runs
const INDEX_RVA: usize = rva(0x755958); // index: the event's stage in its section
const SECTION_RVA: usize = rva(0x75595C); // section
const MODE_RVA: usize = rva(0x7556D0); // dword_7556D0: 0x13 picks the other event loop
const PAYLOAD_OFFSET: usize = 0x30; // 0x51EBE4: what `gather_savegame` wrote

/// The unported functions it calls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Callee {
    Prepare = 0x471F7E,        // sub_471F7E
    CopySomeData = 0x4719F5,   // maybeCopySomeData
    AfterHead = 0x4748B6,      // sub_4748B6
    EventState = 0x470D08,     // sub_470D08: eax = where the event stands
    BeforeEvent = 0x47059E,    // sub_47059E
    RunEventMode13 = 0x4AA4D7, // sub_4AA4D7
    RunEvent = 0x4A0E60,       // sub_4A0E60
    Refuse = 0x40F059,         // sub_40F059, with eax = 8 when the slot cannot be read
}

/// Calls one of them with the registers given, and leaves the registers it returns.
pub type Game<'a> = &'a mut dyn FnMut(Callee, &mut Regs);

fn in_game(callee: Callee, regs: &mut Regs) {
    unsafe { call_game(regs, game_addr(rva(callee as usize))) };
}

/// The slot the highlighted menu item stands for.
fn highlighted_slot() -> u32 {
    unsafe { game_read::<u32>(SAVEGAME_PAGE_RVA) }
        .wrapping_mul(10)
        .wrapping_add(menu_item())
        .wrapping_sub(1)
}

// ---- Rust version ----

/// `entry` holds the registers at entry: the unported callees get the registers the
/// original would have at each call.
pub unsafe fn load_savegame(entry: &Regs, game: Game) -> Result<(), Refused> {
    let slot = highlighted_slot();
    if slot >= SLOTS {
        return Err(BadInput("slot past the name list").into());
    }
    let cars = cars()?;
    let mut g = *entry;

    unsafe {
        game(Callee::Prepare, &mut g);
        game_write(LOADING_RVA, 1u32);
        game_write(CLEARED_A_RVA, 0u32);
        game_write(CLEARED_B_RVA, 0u32);
        g.ebx = slot;
        g.eax = 0;
        if !read_savegame_slot(slot)? {
            g.eax = 8;
            game(Callee::Refuse, &mut g);
            go_back::go_back();
            return Ok(());
        }

        let payload = game_addr(FILE_RVA) + PAYLOAD_OFFSET;
        let mut cursor = payload as *const u8;
        for at in HEAD_RVAS {
            g.eax = (cursor as *const u32).read_unaligned();
            game_write(at, g.eax);
            cursor = cursor.add(4);
        }
        game_write(SAVED_CARS_PTR_RVA, cursor as u32);
        g.esi = cursor as u32;
        if game_read::<u32>(SKIP_COPY_RVA) == 0 {
            game(Callee::CopySomeData, &mut g);
        }
        game(Callee::AfterHead, &mut g);

        for car in cars {
            // The pointer as it is before the car is overwritten: the saved one may be stale.
            let name = ((car + NAME_PTR_OFFSET) as *const u32).read_unaligned();
            copy_nonoverlapping(cursor, car as *mut u8, CAR_LEN);
            std::ptr::copy((car + NAME_OFFSET) as *const u8, name as *mut u8, NAME_LEN);
            cursor = cursor.add(CAR_LEN);
            (g.edi, g.ebp, g.ebx) = (car, name, car + NAME_OFFSET);
        }
        copy_nonoverlapping(cursor, game_addr(RECORDS_RVA) as *mut u8, RECORDS_LEN);
        g.esi = cursor.add(RECORDS_LEN) as u32;
        g.edx = 0xB4;
        g.edi = cars[0];
        game_write(
            CAR0_A_RVA,
            ((cars[0] + 0x14) as *const u32).read_unaligned(),
        );
        g.eax = ((cars[0] + 0x274) as *const u32).read_unaligned();
        game_write(CAR0_B_RVA, g.eax);

        game(Callee::EventState, &mut g);
        match g.eax {
            4 | 5 => {
                game_write(SECTION_RVA, game_read::<u32>(SECTION_RVA).wrapping_add(1));
                game_write(INDEX_RVA, 0u32);
            }
            0 | 1 => game_write(INDEX_RVA, game_read::<u32>(INDEX_RVA).wrapping_add(1)),
            3 => {
                go_back::go_back();
                return Ok(());
            }
            _ => {
                return Err(FatalError {
                    code: 0x8070_2821,
                    message: "Corrupt SaveGame",
                }
                .into());
            }
        }
        game_write(CONTINUING_RVA, 1u32);
        game(Callee::BeforeEvent, &mut g);
        if game_read::<u32>(MODE_RVA) == 0x13 {
            game(Callee::RunEventMode13, &mut g);
        } else {
            game(Callee::RunEvent, &mut g);
        }
        go_back::go_back();
    }
    Ok(())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let entry = *r;
    if let Err(e) = unsafe { load_savegame(&entry, &mut in_game) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test: a copy whose eight calls only note that they were made ----

const FUNCTION_LEN: usize = 0x1B8;
const ENTRY_BYTES: [u8; 6] = [0x60, 0xE8, 0x97, 0x38, 0x00, 0x00]; // pusha; call sub_471F7E
const CALL_OFFSETS: [usize; 11] = [1, 55, 157, 162, 311, 389, 394, 411, 416, 428, 433];

static CALLS: [AtomicU32; 16] = [const { AtomicU32::new(0) }; 16];
static CALL_COUNT: AtomicU32 = AtomicU32::new(0);
static EVENT_STATE: AtomicU32 = AtomicU32::new(0);

/// A stand-in for one callee: writes its address into the list of calls made.
macro_rules! stand_in {
    ($name:ident, $callee:expr $(, $extra:literal)?) => {
        #[unsafe(naked)]
        unsafe extern "C" fn $name() {
            core::arch::naked_asm!(
                "push ecx",
                "mov ecx, [{count}]",
                "and ecx, 15",
                "mov dword ptr [{calls} + ecx * 4], {id}",
                "inc dword ptr [{count}]",
                "pop ecx",
                $($extra,)?
                "ret",
                count = sym CALL_COUNT,
                calls = sym CALLS,
                id = const $callee as u32,
                state = sym EVENT_STATE,
            )
        }
    };
}
stand_in!(fake_prepare, Callee::Prepare, "test dword ptr [{state}], 0");
stand_in!(
    fake_copy_some_data,
    Callee::CopySomeData,
    "test dword ptr [{state}], 0"
);
stand_in!(
    fake_after_head,
    Callee::AfterHead,
    "test dword ptr [{state}], 0"
);
stand_in!(fake_event_state, Callee::EventState, "mov eax, [{state}]");
stand_in!(
    fake_before_event,
    Callee::BeforeEvent,
    "test dword ptr [{state}], 0"
);
stand_in!(
    fake_run_event_13,
    Callee::RunEventMode13,
    "test dword ptr [{state}], 0"
);
stand_in!(
    fake_run_event,
    Callee::RunEvent,
    "test dword ptr [{state}], 0"
);
stand_in!(fake_refuse, Callee::Refuse, "test dword ptr [{state}], 0");

fn noted(callee: Callee, regs: &mut Regs) {
    let n = CALL_COUNT.fetch_add(1, Ordering::SeqCst) & 15;
    CALLS[n as usize].store(callee as u32, Ordering::SeqCst);
    if callee == Callee::EventState {
        regs.eax = EVENT_STATE.load(Ordering::SeqCst);
    }
}

/// Everything a load writes in the game, to compare and to put back.
fn touched(cars: [u32; CARS], slot: u32) -> Vec<Region> {
    let mut regions: Vec<Region> = HEAD_RVAS.map(at).to_vec();
    for rva in [
        LOADING_RVA,
        CLEARED_A_RVA,
        CLEARED_B_RVA,
        SAVED_CARS_PTR_RVA,
        CONTINUING_RVA,
        CAR0_A_RVA,
        CAR0_B_RVA,
        LAST_FILE_PATH_RVA,
    ] {
        regions.push(at(rva));
    }
    regions.extend(go_back::regions());
    for car in cars {
        let name = unsafe { ((car + NAME_PTR_OFFSET) as *const u32).read_unaligned() };
        regions.push((name, NAME_LEN));
        regions.push((car, CAR_LEN));
    }
    regions.push((game_addr(RECORDS_RVA) as u32, RECORDS_LEN));
    regions.push((game_addr(FILE_RVA) as u32, FILE_LEN));
    regions.push((path_buffer() as u32, 0x100));
    if slot < SLOTS {
        regions.push((slot_name(slot) as u32, SLOT_NAME_LEN));
    }
    regions
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let Ok(cars) = cars() else {
        return t.skip("the players are not there");
    };
    let redirect = |target: usize| {
        let stand_in: unsafe extern "C" fn() = match target {
            t if t == rva(Callee::Prepare as usize) => fake_prepare,
            t if t == rva(Callee::CopySomeData as usize) => fake_copy_some_data,
            t if t == rva(Callee::AfterHead as usize) => fake_after_head,
            t if t == rva(Callee::EventState as usize) => fake_event_state,
            t if t == rva(Callee::BeforeEvent as usize) => fake_before_event,
            t if t == rva(Callee::RunEventMode13 as usize) => fake_run_event_13,
            t if t == rva(Callee::RunEvent as usize) => fake_run_event,
            t if t == rva(Callee::Refuse as usize) => fake_refuse,
            _ => return None, // read_savegame_slot, go_back
        };
        Some(stand_in as *const () as usize)
    };
    let copy = match unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x46E6E1)),
            FUNCTION_LEN,
            &ENTRY_BYTES,
            &CALL_OFFSETS,
            &redirect,
        )
    } {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };

    let inputs = [at(SAVEGAME_PAGE_RVA), at(MENU_ITEM_RVA), at(SKIP_COPY_RVA)];
    let saved_inputs = capture(&inputs);
    let observed = || {
        (
            CALLS.each_ref().map(|c| c.load(Ordering::SeqCst)),
            CALL_COUNT.load(Ordering::SeqCst),
        )
    };

    // Slot 0 (the owner's first saved game, if there is one) and slot 99 (none, normally).
    // Only event states the original accepts: any other is a fatal error.
    for (page, item) in [(0u32, 1u32), (9, 10)] {
        unsafe {
            game_write(SAVEGAME_PAGE_RVA, page);
            game_write(MENU_ITEM_RVA, item);
        }
        let regions = touched(cars, highlighted_slot());
        let saved = capture(&regions);
        for state in [0u32, 1, 3, 4, 5] {
            for skip_copy in [0u32, 1] {
                let prepare = || {
                    restore(&regions, &saved);
                    unsafe {
                        game_write(SKIP_COPY_RVA, skip_copy);
                        for rva in [LOADING_RVA, CLEARED_A_RVA, CLEARED_B_RVA, CONTINUING_RVA] {
                            game_write(rva, 0x5EED_0001u32);
                        }
                    }
                    for call in CALLS.iter().chain([&CALL_COUNT]) {
                        call.store(0, Ordering::SeqCst);
                    }
                    EVENT_STATE.store(state, Ordering::SeqCst);
                };

                prepare();
                let before = Regs::SENTINEL;
                let mut ro = before;
                unsafe { call_game(&mut ro, copy.addr()) };
                let theirs = (capture(&regions), observed());

                prepare();
                let outcome = unsafe { load_savegame(&before, &mut noted) };
                let mine = (capture(&regions), observed());

                let preserved = ro.preserved_except(&before, 0);
                t.check(outcome.is_ok() && theirs == mine && preserved, || {
                    let first = mine.0.iter().zip(&theirs.0).position(|(a, b)| a != b);
                    format!(
                        "page={page} item={item} state={state} skip_copy={skip_copy}: preserved={preserved} original calls {:x?}; rust {outcome:?} calls {:x?}; first different byte of the state: {first:x?}",
                        &theirs.1.0[..theirs.1.1.min(16) as usize],
                        &mine.1.0[..mine.1.1.min(16) as usize]
                    )
                });
            }
        }
        restore(&regions, &saved);
    }
    restore(&inputs, &saved_inputs);
    t.finish()
}
