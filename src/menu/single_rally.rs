//! `menu::single_rally` (0x482382): the single rally's event, from the player choice on.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `menu::begin_event`, `clear_players_284`, `take_player1_units`; `dword_755984` 1,
//! `dword_755988` 0x18, `dword_75555D` 0; `clear_event_state`; the current player 0; the car
//! list in esi (`off_759DCA` with `dword_7554D8` 1, else `off_75A042`; `off_75A412` with
//! cheat bit 1); `menu::select_car_loop`, and with `dword_758806` 1 straight back (without
//! the end below); `take_shown_car`. Then each stage: `menu::select_stage_loop` (ended by
//! `g_yesNoAnswer` 1 or `dword_75555D` 1), the current player's +0xD0 1,
//! `service::repair_car`, the race `sub_4843EC` (eax 0, ebx 0); unless `dword_75B028` is -1
//! the current player 0 and `menu::stage_result_loop` (ended by a "yes"); eax 1,
//! `hide_in_game_options`, `menu::review_options_485f5e_loop` (ended by `dword_755A3C`,
//! `dword_755569` or `dword_75555D` 1), and the next. The end: `dword_755980` 1,
//! `dword_75555D`, `dword_755569`, `dword_755565` 0, `show_in_game_options`. Reached from
//! the start menu's table (0x748A8C). The registers each callee gets are the original's
//! (one set carried through, as `pusha` keeps them). Refuses a current player past four.
//! The screens and the race are called at the game's addresses (their hooks choose).
//! Ungated thunk; shadow mode runs the original alone. Self-test: a redirected copy whose
//! variables are the test's and whose 13 calls go to stand-ins that note eax, ebx, esi and
//! act a script (the screens' answers), 9 scripts.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{set_handler, stand_in};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

// The first 5 bytes end inside a `call`: the jump is written by hand and the original runs
// from a private copy with its 13 calls relocated.
pub static HOOK: RawHook = RawHook::new("menu::single_rally", rva(0x482382), thunk, &ORIGINAL)
    .private_copy(LEN)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const LEN: usize = 0x13F;
const CALLS: [usize; 13] = [
    0x01, 0x06, 0x0B, 0x2E, 0x62, 0x74, 0x79, 0xAC, 0xBB, 0xD3, 0xE6, 0xEB, 0x138,
];
/// The callees, by place: the calls' order in the function.
const CALLEES: [usize; 13] = [
    0x472BAA, // 0 begin_event
    0x4A9679, // 1 clear_players_284
    0x467EA8, // 2 take_player1_units
    0x486C2C, // 3 clear_event_state
    0x47AC0C, // 4 select_car_loop
    0x47B29D, // 5 take_shown_car
    0x482099, // 6 select_stage_loop
    0x486F63, // 7 repair_car
    0x4843EC, // 8 the race (not ported)
    0x48558D, // 9 stage_result_loop
    0x46FB63, // 10 hide_in_game_options
    0x485F5E, // 11 review_options_485f5e_loop
    0x46FBE3, // 12 show_in_game_options
];
const LISTS: [usize; 3] = [0x759DCA, 0x75A042, 0x75A412]; // off_759DCA, off_75A042, off_75A412

/// Where its variables are (the game's, or a test's), in the order of `VARIABLES`.
#[derive(Clone, Copy)]
pub struct Vars {
    pub yes_no: u32,
    pub list_kind: u32,
    pub leave: u32,
    pub ended_565: u32,
    pub ended_569: u32,
    pub done: u32,
    pub set_984: u32,
    pub set_988: u32,
    pub current_player: u32,
    pub left_a3c: u32,
    pub players: u32,
    pub cheats: u32,
    pub car_left: u32,
    pub race_result: u32,
}

/// The variables' addresses as IDA shows them, in `Vars`' order.
const VARIABLES: [usize; 14] = [
    0x75524C, // g_yesNoAnswer
    0x7554D8, // dword_7554D8: 1 picks the first car list
    0x75555D, // dword_75555D: leave the event
    0x755565, // dword_755565
    0x755569, // dword_755569
    0x755980, // dword_755980: set to 1 at the end
    0x755984, // dword_755984
    0x755988, // dword_755988
    0x75598C, // dword_75598C: the current player
    0x755A3C, // dword_755A3C
    0x75647C, // g_players
    0x75870C, // g_cheats
    0x758806, // dword_758806: the car choice was left
    0x75B028, // dword_75B028: -1 when the race gave no result
];
/// How often the function's bytes hold each one.
const REFERENCES: usize = 2 + 1 + 4 + 1 + 2 + 1 + 1 + 1 + 3 + 1 + 1 + 1 + 1 + 1;

impl Vars {
    fn at(a: impl Fn(usize) -> u32) -> Self {
        let v = VARIABLES.map(a);
        Self {
            yes_no: v[0],
            list_kind: v[1],
            leave: v[2],
            ended_565: v[3],
            ended_569: v[4],
            done: v[5],
            set_984: v[6],
            set_988: v[7],
            current_player: v[8],
            left_a3c: v[9],
            players: v[10],
            cheats: v[11],
            car_left: v[12],
            race_result: v[13],
        }
    }

    pub fn game() -> Self {
        Self::at(|v| game_addr(rva(v)) as u32)
    }
}

/// A callee by place, with the registers it gets (and leaves).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn put(at: u32, value: u32) {
    unsafe { (at as *mut u32).write_unaligned(value) };
}

pub unsafe fn single_rally(entry: &Regs, v: &Vars, part: Part) -> Result<(), Refused> {
    let mut g = *entry;
    unsafe {
        part(0, &mut g)?;
        part(1, &mut g)?;
        part(2, &mut g)?;
        put(v.set_984, 1);
        put(v.set_988, 0x18);
        put(v.leave, 0);
        part(3, &mut g)?;
        put(v.current_player, 0);
        let list = if get(v.list_kind) == 1 {
            LISTS[0]
        } else {
            LISTS[1]
        };
        let list = if get(v.cheats) & 2 != 0 {
            LISTS[2]
        } else {
            list
        };
        g.esi = game_addr(rva(list)) as u32;
        part(4, &mut g)?;
        if get(v.car_left) == 1 {
            return Ok(());
        }
        part(5, &mut g)?;
        loop {
            part(6, &mut g)?;
            if get(v.yes_no) == 1 || get(v.leave) == 1 {
                break;
            }
            let current = get(v.current_player);
            if current >= 4 {
                return Err(BadInput("a current player past the four").into());
            }
            g.edi = get(v.players + current * 4);
            put(g.edi + 0xD0, 1);
            part(7, &mut g)?;
            g.eax = 0;
            g.ebx = 0;
            part(8, &mut g)?;
            if get(v.race_result) != u32::MAX {
                put(v.current_player, 0);
                part(9, &mut g)?;
                if get(v.yes_no) == 1 {
                    break;
                }
            }
            g.eax = 1;
            part(10, &mut g)?;
            part(11, &mut g)?;
            if get(v.left_a3c) == 1 || get(v.ended_569) == 1 || get(v.leave) == 1 {
                break;
            }
        }
        put(v.done, 1);
        put(v.leave, 0);
        put(v.ended_569, 0);
        put(v.ended_565, 0);
        part(12, &mut g)?;
    }
    Ok(())
}

/// The game's callees: the Rust versions of the small ones, the screens and the race at
/// the game's addresses.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::menu::in_game_options_items as items;
    unsafe {
        match place {
            0 => crate::menu::begin_event::begin_event()?,
            1 => crate::menu::clear_players_284::clear_players_284()?,
            2 => crate::menu::take_player1_units::take_player1_units()?,
            3 => crate::menu::clear_event_state::clear_event_state(),
            5 => crate::menu::take_shown_car::take_shown_car(r.esi)?,
            7 => crate::service::repair_parts::repair_car()?,
            10 => items::hide_in_game_options(r.eax),
            12 => items::show_in_game_options(),
            _ => call_game(r, game_addr(rva(CALLEES[place as usize]))),
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { single_rally(r, &Vars::game(), &mut game_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

/// A run's script: what the stand-ins put in the test's variables, by the callee's place and
/// how many times it was called before.
#[derive(Clone)]
struct Script {
    car_left: u32,
    /// For each stage: the stage choice's yes/no and leave, the race's result, the result
    /// screen's yes/no, the review's three.
    stages: Vec<[u32; 7]>,
}

struct Run {
    vars: Vars,
    script: Script,
    stage: usize,
    noted: Vec<(u32, u32, u32, u32)>,
}

static RUN: Mutex<Option<Run>> = Mutex::new(None);

fn act(place: u32, r: &mut Regs) {
    let mut run = RUN.lock().unwrap();
    let Some(run) = run.as_mut() else { return };
    run.noted.push((place, r.eax, r.ebx, r.esi));
    let v = run.vars;
    // A script that runs out leaves the event at the next stage choice.
    let s = run
        .script
        .stages
        .get(run.stage)
        .copied()
        .unwrap_or([1, 0, 0, 0, 0, 0, 0]);
    unsafe {
        match place {
            4 => put(v.car_left, run.script.car_left),
            6 => {
                put(v.yes_no, s[0]);
                put(v.leave, s[1]);
            }
            8 => put(v.race_result, s[2]),
            9 => put(v.yes_no, s[3]),
            11 => {
                put(v.left_a3c, s[4]);
                put(v.ended_569, s[5]);
                put(v.leave, s[6]);
                run.stage += 1;
            }
            _ => {}
        }
    }
}

fn handler(place: u32, r: &mut Regs) {
    act(place, r);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut mem = Box::new([0u32; 14 + 4 + 4 * 0x40]);
    let at = mem.as_mut_ptr() as u32;
    let players = at + 14 * 4;
    let mut vars = Vars::at(|v| at + 4 * VARIABLES.iter().position(|&x| x == v).unwrap() as u32);
    vars.players = players;
    let player = |n: u32| at + 18 * 4 + n * 0x100;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x482382,
            len: LEN,
            entry: &[0x60, 0xE8, 0x22, 0x08, 0xFF, 0xFF],
            calls: &CALLS,
            jumps_out: &[],
        },
        &|callee| {
            CALLEES
                .iter()
                .position(|&c| rva(c) == callee)
                .map(|n| stand_in(n as u32))
        },
        &VARIABLES.map(|v| (v, vars.at_of(v))),
        REFERENCES,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    let go_on = [0, 0, 0, 0, 0, 0, 0];
    let scripts: Vec<(u32, u32, u32, Script)> = vec![
        (
            1,
            0,
            0,
            Script {
                car_left: 1,
                stages: vec![],
            },
        ),
        (
            1,
            0,
            0,
            Script {
                car_left: 0,
                stages: vec![[1, 0, 0, 0, 0, 0, 0]],
            },
        ),
        (
            0,
            0,
            0,
            Script {
                car_left: 0,
                stages: vec![[0, 1, 0, 0, 0, 0, 0]],
            },
        ),
        (
            0,
            2,
            1,
            Script {
                car_left: 0,
                stages: vec![go_on, go_on, [0, 0, 0, 0, 1, 0, 0]],
            },
        ),
        (
            1,
            0,
            2,
            Script {
                car_left: 0,
                stages: vec![[0, 0, u32::MAX, 0, 0, 0, 0], [0, 0, 0, 1, 0, 0, 0]],
            },
        ),
        (
            0,
            2,
            3,
            Script {
                car_left: 0,
                stages: vec![[0, 0, 5, 0, 0, 1, 0]],
            },
        ),
        (
            0,
            0,
            0,
            Script {
                car_left: 0,
                stages: vec![[0, 0, u32::MAX, 0, 0, 0, 1]],
            },
        ),
        (
            1,
            2,
            1,
            Script {
                car_left: 2,
                stages: vec![go_on, [0, 0, 0, 1, 0, 0, 0]],
            },
        ),
        (
            0,
            0,
            0,
            Script {
                car_left: 0,
                stages: vec![go_on, go_on, go_on],
            },
        ),
    ];
    for (n, (kind, cheats, current, script)) in scripts.into_iter().enumerate() {
        let run = |rust: bool| {
            unsafe {
                std::ptr::write_bytes(at as *mut u8, 0, mem_len());
                put(vars.list_kind, kind);
                put(vars.cheats, cheats);
                put(vars.current_player, current);
                for p in 0..4 {
                    put(players + 4 * p, player(p));
                }
            }
            *RUN.lock().unwrap() = Some(Run {
                vars,
                script: script.clone(),
                stage: 0,
                noted: Vec::new(),
            });
            let before = Regs::SENTINEL;
            let mut ro = before;
            let ok = if rust {
                unsafe {
                    single_rally(&before, &vars, &mut |p, r| {
                        act(p, r);
                        Ok(())
                    })
                }
                .is_ok()
            } else {
                unsafe { call_game(&mut ro, copy.addr()) };
                ro.preserved_except(&before, 0)
            };
            let run = RUN.lock().unwrap().take().unwrap();
            let state =
                unsafe { std::slice::from_raw_parts(at as *const u32, mem_len() / 4) }.to_vec();
            (ok, run.noted, state)
        };
        let theirs = run(false);
        let mine = run(true);
        t.check(theirs.0 && theirs == mine, || {
            format!("script {n}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    drop(mem);
    t.finish()
}

fn mem_len() -> usize {
    (14 + 4 + 4 * 0x40) * 4
}

impl Vars {
    /// The test's address of a variable (by its address as IDA shows it).
    fn at_of(&self, v: usize) -> u32 {
        let all = [
            self.yes_no,
            self.list_kind,
            self.leave,
            self.ended_565,
            self.ended_569,
            self.done,
            self.set_984,
            self.set_988,
            self.current_player,
            self.left_a3c,
            self.players,
            self.cheats,
            self.car_left,
            self.race_result,
        ];
        all[VARIABLES.iter().position(|&x| x == v).unwrap()]
    }
}
