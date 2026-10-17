//! `race::single_rally_race` (0x4843EC, `single_rally`'s race, IDA 2026-10-17): the chosen
//! track looked up (`g_trakPakHeader`), the settings and the players' records
//! (`g_players`) copied into the front end's race variables, the race run
//! (`race::race_from_menu`, with the sound objects and the screen modes round it), and the
//! results copied back into the players' records. Its 902 instructions are a few kinds
//! (copies through eax / al with edi a record, compares and jumps, calls, `rep movsb`,
//! two divisions, one `fld` / `fistp`); they are a table (`Op`, made from the
//! disassembly by script) that `run` does as the original does, with its own registers.

use crate::errors::{FatalError, Refused};
use crate::framework::{Regs, game_addr, rva};

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// A register of the table's.
#[derive(Clone, Copy, Debug)]
pub enum R {
    Eax,
    Ebx,
    Ecx,
    Edx,
    Esi,
    Edi,
    Ebp,
    /// eax's low byte.
    Al,
}

/// One instruction (addresses as IDA shows them; jump targets are op numbers).
#[derive(Clone, Copy, Debug)]
pub enum Op {
    Pusha,
    Popa,
    Push(R),
    Pop(R),
    /// `mov [m], r`.
    Store(R, usize),
    /// `mov r, [m]`.
    Load(R, usize),
    /// `mov r, [edi + n]` (al: a byte).
    FromEdi(R, u32),
    /// `mov [edi + n], r`.
    ToEdi(R, u32),
    /// `mov dword [m], value`.
    Put(usize, u32),
    /// `mov dword [edi + n], value`.
    EdiPut(u32, u32),
    /// `mov edi, [table + eax * 4]`.
    EdiFromTable(usize),
    /// `mov r, value`.
    Imm(R, u32),
    /// `lea r, m`.
    Lea(R, usize),
    /// `lea r, [edi + n]`.
    LeaEdi(R, u32),
    /// `mov a, b`.
    Move(R, R),
    Nop,
    /// `cmp dword [m], value`.
    CmpMem(usize, u32),
    /// `cmp r, value`.
    CmpReg(R, u32),
    /// `cmp dword [edi + n], value`.
    CmpEdi(u32, u32),
    /// `cmp [edi + n], eax`.
    CmpEdiEax(u32),
    Jz(usize),
    Jnz(usize),
    Jmp(usize),
    /// `jz` to a fatal stub (0 "Unknown Shaun TrakNum", 1 the division's).
    JzFatal(usize),
    Call(usize),
    /// `rep movsb`.
    RepMovsb,
    /// `add r, [m]`.
    AddMem(R, usize),
    /// `add r, value`.
    AddImm(R, u32),
    /// `add a, b`.
    AddReg(R, R),
    /// `imul r, value`.
    MulImm(R, u32),
    /// `div ebp` (edx:eax).
    DivEbp,
    /// `push eax; fld [esp]; fistp [esp]; pop ebx`: ebx the float eax rounded.
    RoundEaxToEbx,
    Return,
}

use Op::*;

const FATALS: [FatalError; 2] = [
    FatalError {
        code: 0x8073_9281,
        message: "Unknown Shaun TrakNum",
    },
    FatalError {
        code: 0x8070_1010,
        message: "",
    },
];

/// The calls the table makes, by place (its index + 1), as IDA shows them.
pub const CALLS: [usize; 14] = [
    0x48C997, // race::copy_rally_times
    0x40F826, // sound::stop_music
    0x404830, // race::set_race_screen_mode
    0x47CEA8, // menu::show_loading_picture
    0x415C93, // race::set_up_race_sound_objects
    0x40F02E, // sound::set_up_race_sounds
    0x485570, // race::race_from_menu
    0x404823, // gfx::set_menu_screen_mode (platform)
    0x415C6D, // race::clear_race_sound_objects
    0x40EFF3, // sound::apply_settings
    0x442AF6, // scene::set_up_engine
    0x474E04, // scene::set_up_front_end_scene
    0x407F95, // input::take_key
    0x40F780, // sound::play_tune_3
];

fn mem(a: usize) -> *mut u32 {
    game_addr(rva(a)) as *mut u32
}

fn reg(g: &mut Regs, r: R) -> &mut u32 {
    match r {
        R::Eax | R::Al => &mut g.eax,
        R::Ebx => &mut g.ebx,
        R::Ecx => &mut g.ecx,
        R::Edx => &mut g.edx,
        R::Esi => &mut g.esi,
        R::Edi => &mut g.edi,
        R::Ebp => &mut g.ebp,
    }
}

/// The table from the first op, with eax and ebx the caller's (the run's start and kind).
/// Leaves the registers as it was entered (`pusha` .. `popa`).
pub unsafe fn run(ops: &[Op], entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    let mut stack: Vec<u32> = Vec::new();
    let mut saved: Vec<Regs> = Vec::new();
    let mut zero = false;
    let mut pc = 0;
    unsafe {
        loop {
            let op = ops[pc];
            pc += 1;
            match op {
                Pusha => saved.push(g),
                Popa => {
                    let s = saved.pop().expect("a popa after its pusha");
                    g = Regs {
                        eflags: g.eflags,
                        ..s
                    };
                }
                Push(r) => stack.push(*reg(&mut g, r)),
                Pop(r) => *reg(&mut g, r) = stack.pop().expect("a pop after its push"),
                Store(R::Al, a) => *(mem(a) as *mut u8) = g.eax as u8,
                Store(r, a) => mem(a).write_unaligned(*reg(&mut g, r)),
                Load(r, a) => *reg(&mut g, r) = mem(a).read_unaligned(),
                FromEdi(R::Al, n) => {
                    g.eax = g.eax & !0xFF | *((g.edi.wrapping_add(n)) as *const u8) as u32
                }
                FromEdi(r, n) => {
                    *reg(&mut g, r) = ((g.edi.wrapping_add(n)) as *const u32).read_unaligned()
                }
                ToEdi(r, n) => {
                    let v = *reg(&mut g, r);
                    ((g.edi.wrapping_add(n)) as *mut u32).write_unaligned(v)
                }
                Put(a, v) => mem(a).write_unaligned(v),
                EdiPut(n, v) => ((g.edi.wrapping_add(n)) as *mut u32).write_unaligned(v),
                EdiFromTable(t) => {
                    let at = (game_addr(rva(t)) as u32).wrapping_add(g.eax.wrapping_mul(4));
                    g.edi = (at as *const u32).read_unaligned();
                }
                Imm(r, v) => *reg(&mut g, r) = v,
                Lea(r, a) => *reg(&mut g, r) = game_addr(rva(a)) as u32,
                LeaEdi(r, n) => *reg(&mut g, r) = g.edi.wrapping_add(n),
                Move(a, b) => *reg(&mut g, a) = *reg(&mut g, b),
                Nop => {}
                CmpMem(a, v) => zero = mem(a).read_unaligned() == v,
                CmpReg(r, v) => zero = *reg(&mut g, r) == v,
                CmpEdi(n, v) => {
                    zero = ((g.edi.wrapping_add(n)) as *const u32).read_unaligned() == v
                }
                CmpEdiEax(n) => {
                    zero = ((g.edi.wrapping_add(n)) as *const u32).read_unaligned() == g.eax
                }
                Jz(t) => {
                    if zero {
                        pc = t
                    }
                }
                Jnz(t) => {
                    if !zero {
                        pc = t
                    }
                }
                Jmp(t) => pc = t,
                JzFatal(n) => {
                    if zero {
                        return Err(FATALS[n].into());
                    }
                }
                Call(a) => {
                    let n = CALLS
                        .iter()
                        .position(|&c| c == a)
                        .expect("a call the table has");
                    part(n as u32 + 1, &mut g)?;
                }
                RepMovsb => {
                    std::ptr::copy(g.esi as *const u8, g.edi as *mut u8, g.ecx as usize);
                    g.esi = g.esi.wrapping_add(g.ecx);
                    g.edi = g.edi.wrapping_add(g.ecx);
                    g.ecx = 0;
                }
                AddMem(r, a) => {
                    let v = mem(a).read_unaligned();
                    let x = reg(&mut g, r);
                    *x = x.wrapping_add(v);
                }
                AddImm(r, v) => {
                    let x = reg(&mut g, r);
                    *x = x.wrapping_add(v);
                }
                AddReg(a, b) => {
                    let v = *reg(&mut g, b);
                    let x = reg(&mut g, a);
                    *x = x.wrapping_add(v);
                }
                MulImm(r, v) => {
                    let x = reg(&mut g, r);
                    *x = x.wrapping_mul(v);
                }
                DivEbp => {
                    let n = ((g.edx as u64) << 32) | g.eax as u64;
                    let q = n / g.ebp as u64;
                    g.edx = (n % g.ebp as u64) as u32;
                    g.eax = q as u32;
                }
                RoundEaxToEbx => g.ebx = crate::math::x87::to_int(g.eax) as u32,
                Return => return Ok(g),
            }
        }
    }
}

/// `race::single_rally_race` (0x4843EC): `SINGLE_RALLY`. eax the run's start (1 for none
/// of the loading picture after), ebx the kind (3: the rally's times copied first).
/// Refuses, before anything, a missing player record (`g_players`[1] or
/// [`dword_75598C`]; the original reads and writes through it).
pub unsafe fn single_rally_race(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe {
        if player(1) == 0 || player(mem(0x75598C).read_unaligned()) == 0 {
            return Err(crate::errors::BadInput("a player's record missing").into());
        }
        run(&SINGLE_RALLY, entry, part)
    }
}

/// `g_players`[n].
pub unsafe fn player(n: u32) -> u32 {
    unsafe {
        ((game_addr(rva(0x75647C)) as u32).wrapping_add(n.wrapping_mul(4)) as *const u32)
            .read_unaligned()
    }
}

/// A player's record's length (0x28C apart).
pub const PLAYER_LEN: usize = 0x28C;

/// What the table writes outside the players' records: every `Store` / `Put` and the
/// `rep movsb` destinations (0x75B0DC 0x20, 0x75B09C 0x40, 0x75B02C 0x40).
pub fn written() -> Vec<crate::service::harness::Region> {
    let mut v: Vec<usize> = SINGLE_RALLY
        .iter()
        .filter_map(|op| match *op {
            Store(_, a) | Put(a, _) => Some(a),
            _ => None,
        })
        .collect();
    v.sort();
    v.dedup();
    let mut r: Vec<crate::service::harness::Region> =
        v.iter().map(|&a| (game_addr(rva(a)) as u32, 4)).collect();
    r.extend(
        [(0x75B0DC, 0x20), (0x75B09C, 0x40), (0x75B02C, 0x40)]
            .map(|(a, n)| (game_addr(rva(a)) as u32, n)),
    );
    r
}

/// The calls in the game: the race's Rust versions; the sound, scene, input and screen
/// ones through their hooks (`call_game`; the screen mode is the platform's).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::race::loadlevel::or_in_game;
    let address = CALLS[place as usize - 1];
    or_in_game(address, r, |r| unsafe {
        match place {
            1 => *r = crate::race::rally::copy_rally_times(r, &mut crate::race::rally::log_part)?,
            3 => crate::race::race_shell::set_race_screen_mode(
                r,
                &mut crate::race::race_shell::game_screen_mode,
            ),
            5 => {
                *r = crate::race::sound_tables::set_up_race_sound_objects(
                    r,
                    &crate::race::sound_tables::ObjectVars::game(),
                    &mut crate::race::sound_tables::object_part,
                )?
            }
            7 => {
                *r =
                    crate::race::race_end::race_from_menu(r, &mut crate::race::race_end::menu_part)?
            }
            9 => {
                *r = crate::race::sound_tables::clear_race_sound_objects(
                    r,
                    &mut crate::race::sound_tables::object_part,
                )?
            }
            _ => crate::framework::call_game(r, game_addr(rva(address))),
        }
        Ok(())
    })
}

// The table (made from the disassembly by script).
pub const SINGLE_RALLY: [Op; 902] = [
    Pusha,
    CmpReg(R::Ebx, 0x3),
    Jnz(4),
    Call(0x48C997),
    Load(R::Esi, 0x7556F4),
    Store(R::Esi, 0x75ACD0),
    Store(R::Eax, 0x7587E8),
    Store(R::Ebx, 0x75ACD8),
    Put(0x75B028, 0x0),
    Put(0x75B098, 0x0),
    Put(0x75ACE4, 0x0),
    Put(0x75ACE0, 0x1),
    Call(0x40F826),
    Call(0x404830),
    Call(0x47CEA8),
    Load(R::Eax, 0x7558DC),
    Store(R::Eax, 0x5C2FD0),
    Lea(R::Esi, 0x75ACD0),
    Load(R::Eax, 0x75ACD4),
    Lea(R::Edi, 0x734BC6),
    AddImm(R::Edi, 0x34),
    CmpEdiEax(0x34),
    Jz(27),
    AddImm(R::Edi, 0x40),
    CmpEdi(0x0, 0xFFFFFFFF),
    JzFatal(0),
    Jmp(21),
    FromEdi(R::Eax, 0x1C),
    Store(R::Eax, 0x75ACF8),
    FromEdi(R::Eax, 0x20),
    Store(R::Eax, 0x75ACFC),
    Load(R::Eax, 0x755650),
    Store(R::Eax, 0x75AD00),
    Load(R::Eax, 0x75565C),
    Store(R::Eax, 0x75AD0C),
    Load(R::Eax, 0x755660),
    Store(R::Eax, 0x75AD10),
    Load(R::Eax, 0x755664),
    Store(R::Eax, 0x75AD14),
    Load(R::Eax, 0x755668),
    Store(R::Eax, 0x75AD18),
    Push(R::Edi),
    Push(R::Eax),
    Imm(R::Eax, 0x1),
    EdiFromTable(0x75647C),
    Pop(R::Eax),
    FromEdi(R::Eax, 0x2C),
    Store(R::Eax, 0x75AF94),
    FromEdi(R::Eax, 0x30),
    Store(R::Eax, 0x75AF98),
    FromEdi(R::Eax, 0x34),
    Store(R::Eax, 0x75AF9C),
    FromEdi(R::Eax, 0x38),
    Store(R::Eax, 0x75AFA0),
    FromEdi(R::Eax, 0x3C),
    Store(R::Eax, 0x75AFA4),
    FromEdi(R::Eax, 0x40),
    Store(R::Eax, 0x75AFA8),
    FromEdi(R::Eax, 0x44),
    Store(R::Eax, 0x75AFAC),
    FromEdi(R::Eax, 0x48),
    Store(R::Eax, 0x75AFB0),
    FromEdi(R::Eax, 0x50),
    Store(R::Eax, 0x75AFB8),
    FromEdi(R::Eax, 0x54),
    Store(R::Eax, 0x75AFBC),
    FromEdi(R::Eax, 0x58),
    Store(R::Eax, 0x75AFC0),
    FromEdi(R::Eax, 0x5C),
    Store(R::Eax, 0x75AFC4),
    FromEdi(R::Eax, 0x60),
    Store(R::Eax, 0x75AFC8),
    FromEdi(R::Eax, 0x64),
    Store(R::Eax, 0x75AFCC),
    FromEdi(R::Eax, 0x68),
    Store(R::Eax, 0x75AFD0),
    FromEdi(R::Eax, 0x6C),
    Store(R::Eax, 0x75AFD4),
    FromEdi(R::Eax, 0x70),
    Store(R::Eax, 0x75AFD8),
    FromEdi(R::Eax, 0x74),
    Store(R::Eax, 0x75AFE0),
    FromEdi(R::Eax, 0x78),
    Store(R::Eax, 0x75AFE4),
    FromEdi(R::Al, 0x1C),
    Store(R::Al, 0x75AFE8),
    FromEdi(R::Al, 0x1D),
    Store(R::Al, 0x75AFE9),
    FromEdi(R::Al, 0x1E),
    Store(R::Al, 0x75AFEA),
    FromEdi(R::Al, 0x1F),
    Store(R::Al, 0x75AFEB),
    FromEdi(R::Al, 0x20),
    Store(R::Al, 0x75AFEC),
    FromEdi(R::Al, 0x21),
    Store(R::Al, 0x75AFED),
    FromEdi(R::Al, 0x22),
    Store(R::Al, 0x75AFEE),
    FromEdi(R::Al, 0x23),
    Store(R::Al, 0x75AFEF),
    FromEdi(R::Al, 0x24),
    Store(R::Al, 0x75AFF0),
    FromEdi(R::Al, 0x25),
    Store(R::Al, 0x75AFF1),
    FromEdi(R::Al, 0x26),
    Store(R::Al, 0x75AFF2),
    FromEdi(R::Al, 0x27),
    Store(R::Al, 0x75AFF3),
    Pop(R::Edi),
    CmpMem(0x755988, 0x5),
    Jz(126),
    CmpMem(0x755988, 0x4),
    Jz(126),
    CmpMem(0x755988, 0xF),
    Jz(126),
    CmpMem(0x755988, 0xA),
    Jz(126),
    Push(R::Eax),
    Load(R::Eax, 0x75598C),
    EdiFromTable(0x75647C),
    Pop(R::Eax),
    CmpEdi(0xD0, 0x1),
    Jz(290),
    Put(0x75B028, 0xFFFFFFFF),
    Jmp(897),
    Jmp(290),
    Load(R::Edx, 0x7554C8),
    Store(R::Edx, 0x75ACE4),
    Put(0x75ACE0, 0x2),
    Push(R::Eax),
    Imm(R::Eax, 0x1),
    EdiFromTable(0x75647C),
    Pop(R::Eax),
    CmpEdi(0xD0, 0x1),
    Jz(136),
    Put(0x75B098, 0xFFFFFFFF),
    FromEdi(R::Eax, 0x8),
    Store(R::Eax, 0x75ACEC),
    Load(R::Eax, 0x75ACEC),
    FromEdi(R::Eax, 0x10C),
    Store(R::Eax, 0x75AE88),
    FromEdi(R::Eax, 0x110),
    Store(R::Eax, 0x75AE8C),
    FromEdi(R::Eax, 0x114),
    Store(R::Eax, 0x75AE90),
    FromEdi(R::Eax, 0x118),
    Store(R::Eax, 0x75AE94),
    FromEdi(R::Eax, 0x11C),
    Store(R::Eax, 0x75AE98),
    FromEdi(R::Eax, 0x120),
    Store(R::Eax, 0x75AE9C),
    FromEdi(R::Eax, 0x288),
    Store(R::Eax, 0x75B008),
    Pusha,
    Imm(R::Ecx, 0x20),
    LeaEdi(R::Esi, 0x24C),
    Lea(R::Edi, 0x75B0DC),
    RepMovsb,
    Popa,
    FromEdi(R::Eax, 0x278),
    Store(R::Eax, 0x75B018),
    FromEdi(R::Eax, 0xD4),
    Store(R::Eax, 0x75AEA0),
    FromEdi(R::Eax, 0xD8),
    Store(R::Eax, 0x75AEA4),
    FromEdi(R::Eax, 0xDC),
    Store(R::Eax, 0x75AEA8),
    FromEdi(R::Eax, 0xE0),
    Store(R::Eax, 0x75AEAC),
    FromEdi(R::Eax, 0xE4),
    Store(R::Eax, 0x75AEB0),
    FromEdi(R::Eax, 0xE8),
    Store(R::Eax, 0x75AEB4),
    FromEdi(R::Eax, 0xEC),
    Store(R::Eax, 0x75AEB8),
    FromEdi(R::Eax, 0xF0),
    Store(R::Eax, 0x75AEBC),
    FromEdi(R::Eax, 0xF4),
    Store(R::Eax, 0x75AEC0),
    FromEdi(R::Eax, 0xF8),
    Store(R::Eax, 0x75AEC4),
    FromEdi(R::Eax, 0xFC),
    Store(R::Eax, 0x75AEC8),
    FromEdi(R::Eax, 0x100),
    Store(R::Eax, 0x75AECC),
    FromEdi(R::Eax, 0x104),
    Store(R::Eax, 0x75AEF8),
    FromEdi(R::Eax, 0x108),
    Store(R::Eax, 0x75AED0),
    FromEdi(R::Eax, 0x158),
    Store(R::Eax, 0x75AF64),
    FromEdi(R::Eax, 0x15C),
    Store(R::Eax, 0x75AF68),
    FromEdi(R::Eax, 0x160),
    Store(R::Eax, 0x75AF6C),
    FromEdi(R::Eax, 0x168),
    Store(R::Eax, 0x75AF74),
    FromEdi(R::Eax, 0x164),
    Store(R::Eax, 0x75AF78),
    FromEdi(R::Eax, 0x170),
    Store(R::Eax, 0x75AF80),
    FromEdi(R::Eax, 0x16C),
    Store(R::Eax, 0x75AF7C),
    FromEdi(R::Eax, 0x174),
    Store(R::Eax, 0x75AF84),
    FromEdi(R::Eax, 0x178),
    Store(R::Eax, 0x75AF70),
    FromEdi(R::Eax, 0x284),
    Store(R::Eax, 0x75B010),
    FromEdi(R::Eax, 0x17C),
    Store(R::Eax, 0x75AED4),
    FromEdi(R::Eax, 0x180),
    Store(R::Eax, 0x75AED8),
    FromEdi(R::Eax, 0x184),
    Store(R::Eax, 0x75AEDC),
    FromEdi(R::Eax, 0x188),
    Store(R::Eax, 0x75AEE0),
    FromEdi(R::Eax, 0x18C),
    Store(R::Eax, 0x75AEE4),
    FromEdi(R::Eax, 0x190),
    Store(R::Eax, 0x75AEE8),
    FromEdi(R::Eax, 0x194),
    Store(R::Eax, 0x75AEEC),
    FromEdi(R::Eax, 0x198),
    Store(R::Eax, 0x75AEF0),
    FromEdi(R::Eax, 0x19C),
    Store(R::Eax, 0x75AEF4),
    FromEdi(R::Eax, 0x1A0),
    Store(R::Eax, 0x75AEF8),
    FromEdi(R::Eax, 0x1A4),
    Store(R::Eax, 0x75AEFC),
    FromEdi(R::Eax, 0x1A8),
    Store(R::Eax, 0x75AF00),
    FromEdi(R::Eax, 0x1AC),
    Store(R::Eax, 0x75AF04),
    FromEdi(R::Eax, 0x1B0),
    Store(R::Eax, 0x75AF08),
    FromEdi(R::Eax, 0x1B4),
    Store(R::Eax, 0x75AF0C),
    FromEdi(R::Eax, 0x1B8),
    Store(R::Eax, 0x75AF10),
    FromEdi(R::Eax, 0x1BC),
    Store(R::Eax, 0x75AF14),
    FromEdi(R::Eax, 0x1C0),
    Store(R::Eax, 0x75AF18),
    FromEdi(R::Eax, 0x1C4),
    Store(R::Eax, 0x75AF1C),
    FromEdi(R::Eax, 0x1C8),
    Store(R::Eax, 0x75AF20),
    FromEdi(R::Eax, 0x1CC),
    Store(R::Eax, 0x75AF24),
    FromEdi(R::Eax, 0x1D0),
    Store(R::Eax, 0x75AF28),
    FromEdi(R::Eax, 0x1D4),
    Store(R::Eax, 0x75AF2C),
    FromEdi(R::Eax, 0x1D8),
    Store(R::Eax, 0x75AF30),
    FromEdi(R::Eax, 0x1DC),
    Store(R::Eax, 0x75AF34),
    FromEdi(R::Eax, 0x1E0),
    Store(R::Eax, 0x75AF38),
    FromEdi(R::Eax, 0x1E4),
    Store(R::Eax, 0x75AF3C),
    FromEdi(R::Eax, 0x1E8),
    Store(R::Eax, 0x75AF40),
    FromEdi(R::Eax, 0x1EC),
    Store(R::Eax, 0x75AF44),
    FromEdi(R::Eax, 0x1F0),
    Store(R::Eax, 0x75AF48),
    FromEdi(R::Eax, 0x1F4),
    Store(R::Eax, 0x75AF4C),
    FromEdi(R::Eax, 0x1F8),
    Store(R::Eax, 0x75AF50),
    FromEdi(R::Eax, 0x1FC),
    Store(R::Eax, 0x75AF54),
    FromEdi(R::Eax, 0x200),
    Store(R::Eax, 0x75AF58),
    FromEdi(R::Eax, 0x204),
    Store(R::Eax, 0x75AF5C),
    FromEdi(R::Eax, 0x208),
    Store(R::Eax, 0x75AF60),
    FromEdi(R::Eax, 0x28),
    Store(R::Eax, 0x75AF88),
    FromEdi(R::Eax, 0x154),
    Store(R::Eax, 0x75AF8C),
    FromEdi(R::Eax, 0xC),
    Store(R::Eax, 0x75AF90),
    FromEdi(R::Eax, 0x10),
    Store(R::Eax, 0x75AF90),
    Put(0x75598C, 0x0),
    Push(R::Eax),
    Load(R::Eax, 0x75598C),
    EdiFromTable(0x75647C),
    Pop(R::Eax),
    FromEdi(R::Eax, 0x8),
    Store(R::Eax, 0x75ACE8),
    Load(R::Eax, 0x75ACE8),
    CmpEdi(0xD0, 0x1),
    Jz(300),
    Put(0x75B028, 0xFFFFFFFF),
    FromEdi(R::Eax, 0x10C),
    Store(R::Eax, 0x75AD1C),
    FromEdi(R::Eax, 0x110),
    Store(R::Eax, 0x75AD20),
    FromEdi(R::Eax, 0x114),
    Store(R::Eax, 0x75AD24),
    FromEdi(R::Eax, 0x118),
    Store(R::Eax, 0x75AD28),
    FromEdi(R::Eax, 0x11C),
    Store(R::Eax, 0x75AD2C),
    FromEdi(R::Eax, 0x120),
    Store(R::Eax, 0x75AD30),
    FromEdi(R::Eax, 0x288),
    Store(R::Eax, 0x75B004),
    Pusha,
    Imm(R::Ecx, 0x20),
    LeaEdi(R::Esi, 0x24C),
    Lea(R::Edi, 0x75B06C),
    RepMovsb,
    Popa,
    FromEdi(R::Eax, 0x278),
    Store(R::Eax, 0x75B014),
    FromEdi(R::Eax, 0xD4),
    Store(R::Eax, 0x75AD34),
    FromEdi(R::Eax, 0xD8),
    Store(R::Eax, 0x75AD38),
    FromEdi(R::Eax, 0xDC),
    Store(R::Eax, 0x75AD3C),
    FromEdi(R::Eax, 0xE0),
    Store(R::Eax, 0x75AD40),
    FromEdi(R::Eax, 0xE4),
    Store(R::Eax, 0x75AD44),
    FromEdi(R::Eax, 0xE8),
    Store(R::Eax, 0x75AD48),
    FromEdi(R::Eax, 0xEC),
    Store(R::Eax, 0x75AD4C),
    FromEdi(R::Eax, 0xF0),
    Store(R::Eax, 0x75AD50),
    FromEdi(R::Eax, 0xF4),
    Store(R::Eax, 0x75AD54),
    FromEdi(R::Eax, 0xF8),
    Store(R::Eax, 0x75AD58),
    FromEdi(R::Eax, 0xFC),
    Store(R::Eax, 0x75AD5C),
    FromEdi(R::Eax, 0x100),
    Store(R::Eax, 0x75AD60),
    FromEdi(R::Eax, 0x104),
    Store(R::Eax, 0x75AD8C),
    FromEdi(R::Eax, 0x108),
    Store(R::Eax, 0x75AD64),
    FromEdi(R::Eax, 0x158),
    Store(R::Eax, 0x75ADF8),
    FromEdi(R::Eax, 0x15C),
    Store(R::Eax, 0x75ADFC),
    FromEdi(R::Eax, 0x160),
    Store(R::Eax, 0x75AE00),
    FromEdi(R::Eax, 0x168),
    Store(R::Eax, 0x75AE08),
    FromEdi(R::Eax, 0x164),
    Store(R::Eax, 0x75AE0C),
    FromEdi(R::Eax, 0x170),
    Store(R::Eax, 0x75AE14),
    FromEdi(R::Eax, 0x16C),
    Store(R::Eax, 0x75AE10),
    FromEdi(R::Eax, 0x174),
    Store(R::Eax, 0x75AE18),
    FromEdi(R::Eax, 0x178),
    Store(R::Eax, 0x75AE04),
    FromEdi(R::Eax, 0x284),
    Store(R::Eax, 0x75B00C),
    FromEdi(R::Eax, 0x17C),
    Store(R::Eax, 0x75AD68),
    FromEdi(R::Eax, 0x180),
    Store(R::Eax, 0x75AD6C),
    FromEdi(R::Eax, 0x184),
    Store(R::Eax, 0x75AD70),
    FromEdi(R::Eax, 0x188),
    Store(R::Eax, 0x75AD74),
    FromEdi(R::Eax, 0x18C),
    Store(R::Eax, 0x75AD78),
    FromEdi(R::Eax, 0x190),
    Store(R::Eax, 0x75AD7C),
    FromEdi(R::Eax, 0x194),
    Store(R::Eax, 0x75AD80),
    FromEdi(R::Eax, 0x198),
    Store(R::Eax, 0x75AD84),
    FromEdi(R::Eax, 0x19C),
    Store(R::Eax, 0x75AD88),
    FromEdi(R::Eax, 0x1A0),
    Store(R::Eax, 0x75AD8C),
    FromEdi(R::Eax, 0x1A4),
    Store(R::Eax, 0x75AD90),
    FromEdi(R::Eax, 0x1A8),
    Store(R::Eax, 0x75AD94),
    FromEdi(R::Eax, 0x1AC),
    Store(R::Eax, 0x75AD98),
    FromEdi(R::Eax, 0x1B0),
    Store(R::Eax, 0x75AD9C),
    FromEdi(R::Eax, 0x1B4),
    Store(R::Eax, 0x75ADA0),
    FromEdi(R::Eax, 0x1B8),
    Store(R::Eax, 0x75ADA4),
    FromEdi(R::Eax, 0x1BC),
    Store(R::Eax, 0x75ADA8),
    FromEdi(R::Eax, 0x1C0),
    Store(R::Eax, 0x75ADAC),
    FromEdi(R::Eax, 0x1C4),
    Store(R::Eax, 0x75ADB0),
    FromEdi(R::Eax, 0x1C8),
    Store(R::Eax, 0x75ADB4),
    FromEdi(R::Eax, 0x1CC),
    Store(R::Eax, 0x75ADB8),
    FromEdi(R::Eax, 0x1D0),
    Store(R::Eax, 0x75ADBC),
    FromEdi(R::Eax, 0x1D4),
    Store(R::Eax, 0x75ADC0),
    FromEdi(R::Eax, 0x1D8),
    Store(R::Eax, 0x75ADC4),
    FromEdi(R::Eax, 0x1DC),
    Store(R::Eax, 0x75ADC8),
    FromEdi(R::Eax, 0x1E0),
    Store(R::Eax, 0x75ADCC),
    FromEdi(R::Eax, 0x1E4),
    Store(R::Eax, 0x75ADD0),
    FromEdi(R::Eax, 0x1E8),
    Store(R::Eax, 0x75ADD4),
    FromEdi(R::Eax, 0x1EC),
    Store(R::Eax, 0x75ADD8),
    FromEdi(R::Eax, 0x1F0),
    Store(R::Eax, 0x75ADDC),
    FromEdi(R::Eax, 0x1F4),
    Store(R::Eax, 0x75ADE0),
    FromEdi(R::Eax, 0x1F8),
    Store(R::Eax, 0x75ADE4),
    FromEdi(R::Eax, 0x1FC),
    Store(R::Eax, 0x75ADE8),
    FromEdi(R::Eax, 0x200),
    Store(R::Eax, 0x75ADEC),
    FromEdi(R::Eax, 0x204),
    Store(R::Eax, 0x75ADF0),
    FromEdi(R::Eax, 0x208),
    Store(R::Eax, 0x75ADF4),
    FromEdi(R::Eax, 0x28),
    Store(R::Eax, 0x75AE1C),
    FromEdi(R::Eax, 0x154),
    Store(R::Eax, 0x75AE20),
    FromEdi(R::Eax, 0xC),
    Store(R::Eax, 0x75AE24),
    CmpMem(0x75ACE0, 0x2),
    Jnz(452),
    FromEdi(R::Eax, 0x10),
    Store(R::Eax, 0x75AE24),
    FromEdi(R::Eax, 0x2C),
    Store(R::Eax, 0x75AE28),
    FromEdi(R::Eax, 0x30),
    Store(R::Eax, 0x75AE2C),
    FromEdi(R::Eax, 0x34),
    Store(R::Eax, 0x75AE30),
    FromEdi(R::Eax, 0x38),
    Store(R::Eax, 0x75AE34),
    FromEdi(R::Eax, 0x3C),
    Store(R::Eax, 0x75AE38),
    FromEdi(R::Eax, 0x40),
    Store(R::Eax, 0x75AE3C),
    FromEdi(R::Eax, 0x44),
    Store(R::Eax, 0x75AE40),
    FromEdi(R::Eax, 0x48),
    Store(R::Eax, 0x75AE44),
    FromEdi(R::Eax, 0x50),
    Store(R::Eax, 0x75AE4C),
    FromEdi(R::Eax, 0x54),
    Store(R::Eax, 0x75AE50),
    FromEdi(R::Eax, 0x58),
    Store(R::Eax, 0x75AE54),
    FromEdi(R::Eax, 0x5C),
    Store(R::Eax, 0x75AE58),
    FromEdi(R::Eax, 0x60),
    Store(R::Eax, 0x75AE5C),
    FromEdi(R::Eax, 0x64),
    Store(R::Eax, 0x75AE60),
    FromEdi(R::Eax, 0x68),
    Store(R::Eax, 0x75AE64),
    FromEdi(R::Eax, 0x6C),
    Store(R::Eax, 0x75AE68),
    FromEdi(R::Eax, 0x70),
    Store(R::Eax, 0x75AE6C),
    FromEdi(R::Eax, 0x74),
    Store(R::Eax, 0x75AE74),
    FromEdi(R::Eax, 0x78),
    Store(R::Eax, 0x75AE78),
    FromEdi(R::Al, 0x1C),
    Store(R::Al, 0x75AE7C),
    FromEdi(R::Al, 0x1D),
    Store(R::Al, 0x75AE7D),
    FromEdi(R::Al, 0x1E),
    Store(R::Al, 0x75AE7E),
    FromEdi(R::Al, 0x1F),
    Store(R::Al, 0x75AE7F),
    FromEdi(R::Al, 0x20),
    Store(R::Al, 0x75AE80),
    FromEdi(R::Al, 0x21),
    Store(R::Al, 0x75AE81),
    FromEdi(R::Al, 0x22),
    Store(R::Al, 0x75AE82),
    FromEdi(R::Al, 0x23),
    Store(R::Al, 0x75AE83),
    FromEdi(R::Al, 0x24),
    Store(R::Al, 0x75AE84),
    FromEdi(R::Al, 0x25),
    Store(R::Al, 0x75AE85),
    FromEdi(R::Al, 0x26),
    Store(R::Al, 0x75AE86),
    FromEdi(R::Al, 0x27),
    Store(R::Al, 0x75AE87),
    Put(0x75ACDC, 0x0),
    CmpMem(0x755988, 0x5),
    Jz(526),
    CmpMem(0x755988, 0x4),
    Jz(526),
    CmpMem(0x755988, 0xF),
    Jz(526),
    CmpMem(0x755988, 0xA),
    Jz(526),
    CmpMem(0x755984, 0x1),
    Jz(526),
    Put(0x75ACDC, 0x1),
    Call(0x415C93),
    Call(0x40F02E),
    Call(0x485570),
    Call(0x40F826),
    Call(0x404823),
    Load(R::Eax, 0x7587E8),
    CmpReg(R::Eax, 0x1),
    Jz(535),
    Call(0x47CEA8),
    Call(0x415C6D),
    Call(0x40EFF3),
    Lea(R::Esi, 0x75ACD0),
    CmpMem(0x755988, 0x5),
    Jz(547),
    CmpMem(0x755988, 0x4),
    Jz(547),
    CmpMem(0x755988, 0xF),
    Jz(547),
    CmpMem(0x755988, 0xA),
    Jz(547),
    Jmp(720),
    Push(R::Eax),
    Imm(R::Eax, 0x1),
    EdiFromTable(0x75647C),
    Pop(R::Eax),
    Load(R::Eax, 0x75B010),
    ToEdi(R::Eax, 0x284),
    Load(R::Eax, 0x75B018),
    ToEdi(R::Eax, 0x278),
    Load(R::Eax, 0x75AEA0),
    ToEdi(R::Eax, 0xD4),
    Load(R::Eax, 0x75AEA4),
    ToEdi(R::Eax, 0xD8),
    Load(R::Eax, 0x75AEA8),
    ToEdi(R::Eax, 0xDC),
    Load(R::Eax, 0x75AEAC),
    ToEdi(R::Eax, 0xE0),
    Load(R::Eax, 0x75AEB0),
    ToEdi(R::Eax, 0xE4),
    Load(R::Eax, 0x75AEB4),
    ToEdi(R::Eax, 0xE8),
    Load(R::Eax, 0x75AEB8),
    ToEdi(R::Eax, 0xEC),
    Load(R::Eax, 0x75AEBC),
    ToEdi(R::Eax, 0xF0),
    Load(R::Eax, 0x75AEC0),
    ToEdi(R::Eax, 0xF4),
    Load(R::Eax, 0x75AEC4),
    ToEdi(R::Eax, 0xF8),
    Load(R::Eax, 0x75AEC8),
    ToEdi(R::Eax, 0xFC),
    Load(R::Eax, 0x75AECC),
    ToEdi(R::Eax, 0x100),
    Load(R::Eax, 0x75AEF8),
    ToEdi(R::Eax, 0x104),
    Load(R::Eax, 0x75AED0),
    ToEdi(R::Eax, 0x108),
    Load(R::Eax, 0x75AED4),
    ToEdi(R::Eax, 0x17C),
    Load(R::Eax, 0x75AED8),
    ToEdi(R::Eax, 0x180),
    Load(R::Eax, 0x75AEDC),
    ToEdi(R::Eax, 0x184),
    Load(R::Eax, 0x75AEE0),
    ToEdi(R::Eax, 0x188),
    Load(R::Eax, 0x75AEE4),
    ToEdi(R::Eax, 0x18C),
    Load(R::Eax, 0x75AEE8),
    ToEdi(R::Eax, 0x190),
    Load(R::Eax, 0x75AEEC),
    ToEdi(R::Eax, 0x194),
    Load(R::Eax, 0x75AEF0),
    ToEdi(R::Eax, 0x198),
    Load(R::Eax, 0x75AEF4),
    ToEdi(R::Eax, 0x19C),
    Load(R::Eax, 0x75AEF8),
    ToEdi(R::Eax, 0x1A0),
    Load(R::Eax, 0x75AEFC),
    ToEdi(R::Eax, 0x1A4),
    Load(R::Eax, 0x75AF00),
    ToEdi(R::Eax, 0x1A8),
    Load(R::Eax, 0x75AF04),
    ToEdi(R::Eax, 0x1AC),
    Load(R::Eax, 0x75AF08),
    ToEdi(R::Eax, 0x1B0),
    Load(R::Eax, 0x75AF0C),
    ToEdi(R::Eax, 0x1B4),
    Load(R::Eax, 0x75AF10),
    ToEdi(R::Eax, 0x1B8),
    Load(R::Eax, 0x75AF14),
    ToEdi(R::Eax, 0x1BC),
    Load(R::Eax, 0x75AF18),
    ToEdi(R::Eax, 0x1C0),
    Load(R::Eax, 0x75AF1C),
    ToEdi(R::Eax, 0x1C4),
    Load(R::Eax, 0x75AF20),
    ToEdi(R::Eax, 0x1C8),
    Load(R::Eax, 0x75AF24),
    ToEdi(R::Eax, 0x1CC),
    Load(R::Eax, 0x75AF28),
    ToEdi(R::Eax, 0x1D0),
    Load(R::Eax, 0x75AF2C),
    ToEdi(R::Eax, 0x1D4),
    Load(R::Eax, 0x75AF30),
    ToEdi(R::Eax, 0x1D8),
    Load(R::Eax, 0x75AF34),
    ToEdi(R::Eax, 0x1DC),
    Load(R::Eax, 0x75AF38),
    ToEdi(R::Eax, 0x1E0),
    Load(R::Eax, 0x75AF3C),
    ToEdi(R::Eax, 0x1E4),
    Load(R::Eax, 0x75AF40),
    ToEdi(R::Eax, 0x1E8),
    Load(R::Eax, 0x75AF44),
    ToEdi(R::Eax, 0x1EC),
    Load(R::Eax, 0x75AF48),
    ToEdi(R::Eax, 0x1F0),
    Load(R::Eax, 0x75AF4C),
    ToEdi(R::Eax, 0x1F4),
    Load(R::Eax, 0x75AF50),
    ToEdi(R::Eax, 0x1F8),
    Load(R::Eax, 0x75AF54),
    ToEdi(R::Eax, 0x1FC),
    Load(R::Eax, 0x75AF58),
    ToEdi(R::Eax, 0x200),
    Load(R::Eax, 0x75AF5C),
    ToEdi(R::Eax, 0x204),
    Load(R::Eax, 0x75AF60),
    ToEdi(R::Eax, 0x208),
    Pusha,
    Imm(R::Ecx, 0x20),
    Lea(R::Esi, 0x75B0DC),
    LeaEdi(R::Edi, 0x24C),
    RepMovsb,
    Popa,
    Load(R::Ebx, 0x75AEFC),
    AddMem(R::Ebx, 0x75AF00),
    AddMem(R::Ebx, 0x75AF04),
    AddMem(R::Ebx, 0x75AF08),
    CmpReg(R::Ebx, 0x0),
    Jz(674),
    Move(R::Eax, R::Ebx),
    CmpReg(R::Eax, 0x0),
    JzFatal(1),
    Imm(R::Edx, 0),
    Imm(R::Ebp, 0xA),
    DivEbp,
    Move(R::Ebx, R::Eax),
    Load(R::Eax, 0x75AF0C),
    MulImm(R::Eax, 0x6),
    CmpReg(R::Eax, 0x0),
    Jz(684),
    Nop,
    CmpReg(R::Eax, 0x0),
    JzFatal(1),
    Imm(R::Edx, 0),
    Imm(R::Ebp, 0xA),
    DivEbp,
    AddReg(R::Ebx, R::Eax),
    ToEdi(R::Ebx, 0xD8),
    Load(R::Eax, 0x75AF40),
    ToEdi(R::Eax, 0xF8),
    Load(R::Eax, 0x75AF60),
    ToEdi(R::Eax, 0x108),
    Load(R::Eax, 0x75AF4C),
    ToEdi(R::Eax, 0x100),
    Load(R::Eax, 0x75AF2C),
    ToEdi(R::Eax, 0xE8),
    Load(R::Eax, 0x75AF8C),
    ToEdi(R::Eax, 0x154),
    Load(R::Eax, 0x75AF90),
    ToEdi(R::Eax, 0xC),
    CmpMem(0x75B098, 0xFFFFFFFE),
    Jz(702),
    CmpMem(0x75B098, 0xFFFFFFFF),
    Jnz(705),
    EdiPut(0x88, 0x36EE80),
    EdiPut(0xD0, 0x6),
    Jmp(719),
    Load(R::Eax, 0x75B08C),
    ToEdi(R::Eax, 0x88),
    Load(R::Eax, 0x75B090),
    ToEdi(R::Eax, 0x27C),
    Load(R::Eax, 0x75B094),
    RoundEaxToEbx,
    MulImm(R::Ebx, 0xA),
    ToEdi(R::Ebx, 0x94),
    Pusha,
    Imm(R::Ecx, 0x40),
    Lea(R::Esi, 0x75B09C),
    LeaEdi(R::Edi, 0x20C),
    RepMovsb,
    Popa,
    Put(0x75598C, 0x0),
    Push(R::Eax),
    Load(R::Eax, 0x75598C),
    EdiFromTable(0x75647C),
    Pop(R::Eax),
    Load(R::Eax, 0x75B00C),
    ToEdi(R::Eax, 0x284),
    Pusha,
    Imm(R::Ecx, 0x20),
    Lea(R::Esi, 0x75B06C),
    LeaEdi(R::Edi, 0x24C),
    RepMovsb,
    Popa,
    Load(R::Eax, 0x75B014),
    ToEdi(R::Eax, 0x278),
    Load(R::Eax, 0x75AD34),
    ToEdi(R::Eax, 0xD4),
    Load(R::Eax, 0x75AD38),
    ToEdi(R::Eax, 0xD8),
    Load(R::Eax, 0x75AD3C),
    ToEdi(R::Eax, 0xDC),
    Load(R::Eax, 0x75AD40),
    ToEdi(R::Eax, 0xE0),
    Load(R::Eax, 0x75AD44),
    ToEdi(R::Eax, 0xE4),
    Load(R::Eax, 0x75AD48),
    ToEdi(R::Eax, 0xE8),
    Load(R::Eax, 0x75AD4C),
    ToEdi(R::Eax, 0xEC),
    Load(R::Eax, 0x75AD50),
    ToEdi(R::Eax, 0xF0),
    Load(R::Eax, 0x75AD54),
    ToEdi(R::Eax, 0xF4),
    Load(R::Eax, 0x75AD5C),
    ToEdi(R::Eax, 0xFC),
    Load(R::Eax, 0x75AD60),
    ToEdi(R::Eax, 0x100),
    Load(R::Eax, 0x75AD8C),
    ToEdi(R::Eax, 0x104),
    Load(R::Eax, 0x75AD64),
    ToEdi(R::Eax, 0x108),
    Load(R::Eax, 0x75AD68),
    ToEdi(R::Eax, 0x17C),
    Load(R::Eax, 0x75AD6C),
    ToEdi(R::Eax, 0x180),
    Load(R::Eax, 0x75AD70),
    ToEdi(R::Eax, 0x184),
    Load(R::Eax, 0x75AD74),
    ToEdi(R::Eax, 0x188),
    Load(R::Eax, 0x75AD78),
    ToEdi(R::Eax, 0x18C),
    Load(R::Eax, 0x75AD7C),
    ToEdi(R::Eax, 0x190),
    Load(R::Eax, 0x75AD80),
    ToEdi(R::Eax, 0x194),
    Load(R::Eax, 0x75AD84),
    ToEdi(R::Eax, 0x198),
    Load(R::Eax, 0x75AD88),
    ToEdi(R::Eax, 0x19C),
    Load(R::Eax, 0x75AD8C),
    ToEdi(R::Eax, 0x1A0),
    Load(R::Eax, 0x75AD90),
    ToEdi(R::Eax, 0x1A4),
    Load(R::Eax, 0x75AD94),
    ToEdi(R::Eax, 0x1A8),
    Load(R::Eax, 0x75AD98),
    ToEdi(R::Eax, 0x1AC),
    Load(R::Eax, 0x75AD9C),
    ToEdi(R::Eax, 0x1B0),
    Load(R::Eax, 0x75ADA0),
    ToEdi(R::Eax, 0x1B4),
    Load(R::Eax, 0x75ADA4),
    ToEdi(R::Eax, 0x1B8),
    Load(R::Eax, 0x75ADA8),
    ToEdi(R::Eax, 0x1BC),
    Load(R::Eax, 0x75ADAC),
    ToEdi(R::Eax, 0x1C0),
    Load(R::Eax, 0x75ADB0),
    ToEdi(R::Eax, 0x1C4),
    Load(R::Eax, 0x75ADB4),
    ToEdi(R::Eax, 0x1C8),
    Load(R::Eax, 0x75ADB8),
    ToEdi(R::Eax, 0x1CC),
    Load(R::Eax, 0x75ADBC),
    ToEdi(R::Eax, 0x1D0),
    Load(R::Eax, 0x75ADC0),
    ToEdi(R::Eax, 0x1D4),
    Load(R::Eax, 0x75ADC4),
    ToEdi(R::Eax, 0x1D8),
    Load(R::Eax, 0x75ADC8),
    ToEdi(R::Eax, 0x1DC),
    Load(R::Eax, 0x75ADCC),
    ToEdi(R::Eax, 0x1E0),
    Load(R::Eax, 0x75ADD0),
    ToEdi(R::Eax, 0x1E4),
    Load(R::Eax, 0x75ADD4),
    ToEdi(R::Eax, 0x1E8),
    Load(R::Eax, 0x75ADD8),
    ToEdi(R::Eax, 0x1EC),
    Load(R::Eax, 0x75ADDC),
    ToEdi(R::Eax, 0x1F0),
    Load(R::Eax, 0x75ADE0),
    ToEdi(R::Eax, 0x1F4),
    Load(R::Eax, 0x75ADE4),
    ToEdi(R::Eax, 0x1F8),
    Load(R::Eax, 0x75ADE8),
    ToEdi(R::Eax, 0x1FC),
    Load(R::Eax, 0x75ADEC),
    ToEdi(R::Eax, 0x200),
    Load(R::Eax, 0x75ADF0),
    ToEdi(R::Eax, 0x204),
    Load(R::Eax, 0x75ADF4),
    ToEdi(R::Eax, 0x208),
    Load(R::Ebx, 0x75AD90),
    AddMem(R::Ebx, 0x75AD94),
    AddMem(R::Ebx, 0x75AD98),
    AddMem(R::Ebx, 0x75AD9C),
    CmpReg(R::Ebx, 0x0),
    Jz(845),
    Move(R::Eax, R::Ebx),
    CmpReg(R::Eax, 0x0),
    JzFatal(1),
    Imm(R::Edx, 0),
    Imm(R::Ebp, 0xA),
    DivEbp,
    Move(R::Ebx, R::Eax),
    Load(R::Eax, 0x75ADA0),
    MulImm(R::Eax, 0x6),
    CmpReg(R::Eax, 0x0),
    Jz(855),
    Nop,
    CmpReg(R::Eax, 0x0),
    JzFatal(1),
    Imm(R::Edx, 0),
    Imm(R::Ebp, 0xA),
    DivEbp,
    AddReg(R::Ebx, R::Eax),
    ToEdi(R::Ebx, 0xD8),
    Load(R::Eax, 0x75ADD4),
    ToEdi(R::Eax, 0xF8),
    Load(R::Eax, 0x75ADF4),
    ToEdi(R::Eax, 0x108),
    Load(R::Eax, 0x75ADE0),
    ToEdi(R::Eax, 0x100),
    Load(R::Eax, 0x75ADC0),
    ToEdi(R::Eax, 0xE8),
    Load(R::Eax, 0x75AE20),
    ToEdi(R::Eax, 0x154),
    Load(R::Eax, 0x75AE24),
    ToEdi(R::Eax, 0xC),
    CmpMem(0x75B028, 0xFFFFFFFE),
    Jz(873),
    CmpMem(0x75B028, 0xFFFFFFFF),
    Jnz(876),
    EdiPut(0x88, 0x36EE80),
    EdiPut(0xD0, 0x6),
    Jmp(890),
    Load(R::Eax, 0x75B01C),
    ToEdi(R::Eax, 0x88),
    Load(R::Eax, 0x75B020),
    ToEdi(R::Eax, 0x27C),
    Load(R::Eax, 0x75B024),
    RoundEaxToEbx,
    MulImm(R::Ebx, 0xA),
    ToEdi(R::Ebx, 0x94),
    Pusha,
    Imm(R::Ecx, 0x40),
    Lea(R::Esi, 0x75B02C),
    LeaEdi(R::Edi, 0x20C),
    RepMovsb,
    Popa,
    Load(R::Eax, 0x7587E8),
    CmpReg(R::Eax, 0x1),
    Jz(896),
    Call(0x442AF6),
    Call(0x474E04),
    Call(0x442AF6),
    Call(0x407F95),
    Load(R::Edx, 0x75ACE4),
    Store(R::Edx, 0x7554C8),
    Call(0x40F780),
    Popa,
    Return,
];
