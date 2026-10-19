//! `menu::time_trial_service` (0x48E093): the time trial's service area before its stage.
//! `dword_75593C` 0xA8C (the service's minutes), `dword_75592C` 1; for each counted player
//! (`dword_75598C` 0 .. `dword_755984`, the original's `do .. while` on ecx, ebx 0): the
//! inspection's screen (`menu::inspection_screen`), a second's sleep, the fade in, 250 ms'
//! sleep, `dword_75AC8C` 0, the figures (`menu::driver_stats`), `dword_75AB38` 0, the
//! service page (`service::service_area_loop`; a yes to leaving ends it all), the repairs
//! (`service::apply_repairs`), the entry kind (`menu::set_player_entry_kind`); a player
//! now out (+0xD0 2) gets the faults (`menu::inspection_faults`) and the inspection page
//! (`menu::inspection_area_loop`; a yes ends it all) and counts in `dword_755978`. When
//! every player is out, `dword_75592C` 0. At the end `dword_755938` 0.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! `menu::service_pass` without its due test, its money and its keys. Callers: the time
//! trial's events. The calls through `event_flow::Call` (in the game `game_call`).
//! Ungated (the pages wait for the player). Refuses a player count of 0 or past 4 or a
//! player missing before anything, a current player past the four when it is read.
//! Shadow mode runs the original alone. Self-test: `event_flow::test_flow`, scripts of
//! the two pages' answers and the players' +0xD0.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::menu::event_flow::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::time_trial_service", rva(0x48E093), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const SERVICE_TIME: usize = 0x75593C; // dword_75593C
const GOES_ON: usize = 0x75592C; // dword_75592C
const CLEARED: [usize; 2] = [0x75AC8C, 0x75AB38]; // dword_75AC8C, dword_75AB38
const OUT: usize = 0x755978; // dword_755978
const DUE: usize = 0x755938; // dword_755938

pub unsafe fn time_trial_service(entry: &Regs, call: Call) -> Result<Regs, Refused> {
    crate::menu::standings::counted_players()?;
    let mut g = *entry;
    set(SERVICE_TIME, 0xA8C);
    set(GOES_ON, 1);
    g.ecx = 0;
    g.ebx = 0;
    'all: {
        loop {
            set(CURRENT, g.ecx);
            call(INSPECTION_SCREEN, &mut g)?;
            g.eax = 1000;
            call(SLEEP, &mut g)?;
            call(FADE_IN, &mut g)?;
            g.eax = 0xFA;
            call(SLEEP, &mut g)?;
            set(CLEARED[0], 0);
            call(DRIVER_STATS, &mut g)?;
            set(CLEARED[1], 0);
            call(SERVICE_AREA_LOOP, &mut g)?;
            if yes() {
                break 'all;
            }
            call(APPLY_REPAIRS, &mut g)?;
            call(SET_PLAYER_ENTRY_KIND, &mut g)?;
            let current = get(CURRENT);
            if current >= 4 || player(current) == 0 {
                return Err(BadInput("a current player past the four or missing").into());
            }
            g.edi = player(current);
            if dword(g.edi + 0xD0) == 2 {
                call(INSPECTION_FAULTS, &mut g)?;
                call(INSPECTION_AREA_LOOP, &mut g)?;
                if yes() {
                    break 'all;
                }
                inc(OUT);
            }
            g.ecx = g.ecx.wrapping_add(1);
            if g.ecx == get(PLAYER_COUNT) {
                break;
            }
        }
        if get(OUT) == get(PLAYER_COUNT) {
            set(GOES_ON, 0);
        }
    }
    set(DUE, 0);
    Ok(*entry)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { time_trial_service(r, &mut game_call) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

const CALLS: [usize; 10] = [0x25, 0x2F, 0x34, 0x3E, 0x4D, 0x5C, 0x6A, 0x6F, 0x8B, 0x90];

fn self_test() -> (usize, usize) {
    let Some(t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if (0..4).any(|n| player(n) == 0) {
        return t.skip("a player missing from g_players");
    }
    let flow = Flow {
        address: 0x48E093,
        len: 0xD6,
        // pusha; mov dword_75593C, 0A8Ch
        entry: &[
            0x60, 0xC7, 0x05, 0x3C, 0x59, 0x75, 0x00, 0x8C, 0x0A, 0x00, 0x00,
        ],
        calls: &CALLS,
        fatal_jumps: &[],
        writes: &[
            SERVICE_TIME,
            GOES_ON,
            CLEARED[0],
            CLEARED[1],
            OUT,
            DUE,
            CURRENT,
            PLAYER_COUNT,
        ],
    };
    // A player out after the repairs: the entry kind's answer writes its +0xD0 (the
    // test's players are the game's).
    let out = |n: u32| Answer::none().and_player(n, 0xD0, 2);
    let mut cases = Vec::new();
    for (count, script) in [
        (1u32, vec![]),
        (2, vec![(SET_PLAYER_ENTRY_KIND, vec![out(0), out(1)])]),
        (
            3,
            vec![(SET_PLAYER_ENTRY_KIND, vec![Answer::none(), out(1)])],
        ),
        (
            4,
            vec![(
                SERVICE_AREA_LOOP,
                vec![Answer::none(), Answer::none(), Answer::yes()],
            )],
        ),
        (
            2,
            vec![
                (SET_PLAYER_ENTRY_KIND, vec![out(0)]),
                (INSPECTION_AREA_LOOP, vec![Answer::yes()]),
            ],
        ),
    ] {
        cases.push((count, script));
    }
    let prepares: Vec<Box<dyn Fn()>> = cases
        .iter()
        .map(|&(count, _)| {
            Box::new(move || {
                set(PLAYER_COUNT, count);
                set(OUT, 0);
                set(ANSWER, 0);
                for n in 0..4 {
                    put(player(n) + 0xD0, 1);
                }
            }) as Box<dyn Fn()>
        })
        .collect();
    let cases: Vec<Case> = cases
        .into_iter()
        .zip(&prepares)
        .enumerate()
        .map(|(n, ((count, script), prepare))| Case {
            what: format!("case {n}, {count} players"),
            prepare: prepare.as_ref(),
            script,
            before: Regs::SENTINEL,
        })
        .collect();
    test_flow(t, &flow, &[], &cases, &|r, c| unsafe {
        time_trial_service(r, c)
    })
}
