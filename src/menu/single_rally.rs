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
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use rc2k_core::menu::flow::{self, CarList, Hand, Reply, Var};

// The first 5 bytes end inside a `call`: the jump is written by hand and the original runs
// from a private copy with its 13 calls relocated.
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

/// The game for `rc2k_core::menu::rounds::single_rally`: the variables where `Vars` puts
/// them, the callees by place.
struct Rally<'a> {
    v: Vars,
    g: Regs,
    part: Part<'a>,
}

impl Rally<'_> {
    fn at(&self, var: Var) -> u32 {
        let v = &self.v;
        match var {
            Var::Answer => v.yes_no,
            Var::CarListKind => v.list_kind,
            Var::Leave => v.leave,
            Var::Ended565 => v.ended_565,
            Var::Ended569 => v.ended_569,
            Var::Done => v.done,
            Var::PlayerCount => v.set_984,
            Var::EventMode => v.set_988,
            Var::Current => v.current_player,
            Var::LeftA3C => v.left_a3c,
            Var::Cheats => v.cheats,
            Var::CarLeft => v.car_left,
            Var::RaceResult => v.race_result,
            _ => unreachable!("a variable the single rally does not have"),
        }
    }
}

impl flow::Game for Rally<'_> {
    type Error = Refused;

    fn get(&mut self, var: Var) -> u32 {
        unsafe { get(self.at(var)) }
    }

    fn set(&mut self, var: Var, value: u32) {
        unsafe {
            match var {
                Var::PlayerEntry(n) => {
                    self.g.edi = get(self.v.players + n * 4);
                    put(self.g.edi + 0xD0, value);
                }
                _ => put(self.at(var), value),
            }
        }
    }

    fn call(&mut self, c: flow::Call) -> Result<Reply, Refused> {
        use flow::Call as C;
        let place = match c {
            C::BeginEvent => 0,
            C::ClearPlayers284 => 1,
            C::TakePlayer1Units => 2,
            C::ClearEventState => 3,
            C::SelectCarLoop => 4,
            C::TakeShownCar => 5,
            C::SelectStageLoop => 6,
            C::RepairCar { .. } => 7,
            C::Race { next, kind } => {
                self.g.eax = u32::from(next);
                self.g.ebx = kind;
                8
            }
            C::StageResultLoop => 9,
            C::HideInGameOptions { players } => {
                self.g.eax = players;
                10
            }
            C::Review485F5E => 11,
            C::ShowInGameOptions { .. } => 12,
            _ => unreachable!("a callee the single rally does not have"),
        };
        (self.part)(place, &mut self.g)?;
        Ok(Reply {
            eax: self.g.eax,
            carry: self.g.carry(),
            zero: self.g.eflags & 0x40 != 0,
        })
    }

    fn hand(&mut self, h: Hand) -> Result<(), Refused> {
        match h {
            Hand::CarList(list) => {
                self.g.esi = game_addr(rva(match list {
                    CarList::First => LISTS[0],
                    CarList::Second => LISTS[1],
                    CarList::Cheat => LISTS[2],
                })) as u32
            }
            _ => unreachable!("a hand the single rally does not give"),
        }
        Ok(())
    }

    fn fatal(&mut self, code: u32, message: &'static str) -> Refused {
        crate::errors::FatalError { code, message }.into()
    }
}

pub unsafe fn single_rally(entry: &Regs, v: &Vars, part: Part) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::rounds::single_rally`.
    rc2k_core::menu::rounds::single_rally(&mut Rally {
        v: *v,
        g: *entry,
        part,
    })
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
            _ => crate::calls::call(rva(CALLEES[place as usize]), r),
        }
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { single_rally(r, &Vars::game(), &mut game_part) }?;
    Ok(())
}

impl Vars {}
