//! `install::run_after_exit` (0x412FA2): what the installer's pages asked to be done once
//! the game has closed (not after a fatal error, `g_fatalReported` 1):
//! - `dword_51B9D4` 1: the Start menu's items added (`sub_413E42`, DDE to the Program
//!   Manager: "Rally", "Uninstall Rally", "View Readme"); `dword_51B9D8` 1: removed
//!   (`sub_4140C3`, DDE);
//! - then the first of: `dword_51B9DC` 1: DirectX's set-up run (into the game's folder
//!   `byte_95E6F8`, then `unk_5295B5`; `sub_410ADC` "redist\directx\dxsetup.exe", not
//!   waited for); `dword_51B9E8` 1: the registration program run (its folder, the game's
//!   and "ereg\", made current; "eregip.exe" into `byte_94E8B0`); installing
//!   (`g_installing` 1): the installed folder (`g_installSource` normalised into
//!   `byte_95E7F8`) made current, from the CD (`g_runningFromCd` 1) the program named on
//!   the command line (`dword_518BA8`, `dword_518BAC` bytes, into `byte_94E8B0`) copied
//!   to ".\ral.exe" (`files::copy_file`) and a second's sleep, and ".\ral.exe" run.
//!
//! A folder that cannot be made current ends it.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! One caller, `start`'s way out. Its calls through `part`: the DDE and the programs' start
//! through the game (platform, section 15), `set_current_directory` (`SetCurrentDirectoryA`),
//! `normalise_path`, `copy_file`, `sleep` the Rust versions; `text_length`,
//! `append_directory`, `append_nul`, `append_file_name` written in. Refuses (before
//! anything, where the way could lead there) a game folder too long for "ereg\" in its
//! 0x100 bytes, and a command line's program of 0x100 bytes or more. Shadow mode runs the
//! original alone (programs are started). Self-test: a copy whose eighteen references to
//! the variables go to the test's and whose calls go to noting stand-ins, against the Rust
//! version: each switch, from the CD or not, after a fatal error (the stand-ins leave the
//! carry clear: the failures' ways are not reached).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::input::effects::compare;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::run_after_exit", rva(0x412FA2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

// The places.
const ADD_ITEMS: u32 = 0x413E42; // install::add_start_menu_items (DDE)
const REMOVE_ITEMS: u32 = 0x4140C3; // install::remove_start_menu_items (DDE)
const SET_CD: u32 = 0x403105; // files::set_current_directory (edx; CF failed)
const RUN: u32 = 0x410ADC; // system::run_program (eax the command, ebx 1: not waited for)
const NORMALISE: u32 = 0x4139EC; // install::normalise_path (esi into edx; CF not written)
const COPY_FILE: u32 = 0x402C28; // files::copy_file (eax to ebx)
const SLEEP: u32 = 0x41100C; // system::sleep (eax)

// The variables.
const REPORTED: usize = 0x4F67BC; // g_fatalReported (a byte)
const SWITCHES: usize = 0x51B9D4; // dword_51B9D4, D8, DC, g_installing (E0), E8
const GAME_DIR: usize = 0x95E6F8; // byte_95E6F8: the game's folder, 0x100 bytes
const INSTALLED: usize = 0x95E7F8; // byte_95E7F8: the installed folder, 0x100 bytes
const OTHER_DIR: usize = 0x5295B5; // unk_5295B5
const PROGRAM: usize = 0x94E8B0; // byte_94E8B0: a program's name (0x100 bytes taken)
const SOURCE: usize = 0x51BB8D; // g_installSource
const FROM_CD: usize = 0x529DB0; // g_runningFromCd
const COMMAND: usize = 0x518BA8; // dword_518BA8 the program's name, dword_518BAC its length
// Texts.
const DXSETUP: usize = 0x51BE54; // "redist\directx\dxsetup.exe"
const EREG: usize = 0x51B9EE; // "ereg\eregip.exe"
const RAL: usize = 0x51BE3E; // ".\ral.exe"

const ROOM: u32 = 0x100;

/// Where the variables are.
#[derive(Clone, Copy)]
pub struct Vars {
    pub reported: u32,
    pub switches: u32,
    pub game_dir: u32,
    pub installed: u32,
    pub other_dir: u32,
    pub program: u32,
    pub source: u32,
    pub from_cd: u32,
    pub command: u32,
}

impl Vars {
    pub fn game() -> Self {
        let a = |x: usize| game_addr(rva(x)) as u32;
        Self {
            reported: a(REPORTED),
            switches: a(SWITCHES),
            game_dir: a(GAME_DIR),
            installed: a(INSTALLED),
            other_dir: a(OTHER_DIR),
            program: a(PROGRAM),
            source: a(SOURCE),
            from_cd: a(FROM_CD),
            command: a(COMMAND),
        }
    }
}

fn rd(at: u32) -> u32 {
    unsafe { (at as *const u32).read_volatile() }
}

/// A call whose failure is its carry (the original's `jb`): the carry is clear before it,
/// as the `cmp` before each call leaves it.
fn checked(place: u32, g: &mut Regs, part: Part) -> Result<bool, Refused> {
    g.set_carry(false);
    part(place, g)?;
    Ok(!g.carry())
}

/// The Rust version on the variables in `v`.
pub fn run_after_exit_with(r: &mut Regs, v: &Vars, part: Part) -> Result<(), Refused> {
    use crate::files::append_directory::append_directory;
    use crate::files::append_file_name::append_file_name;
    let text = |x: usize| game_addr(rva(x)) as u32;
    let switch = |n: u32| rd(v.switches + 4 * n);
    if unsafe { (v.reported as *const u8).read_volatile() } == 1 {
        return Ok(());
    }
    // The refusals, before anything.
    let game_dir_len =
        unsafe { crate::strings::text_length::text_length(v.game_dir as *const u8) }?;
    if switch(2) != 1 && switch(5) == 1 && game_dir_len + 5 + 1 > ROOM {
        return Err(
            BadInput("the game's folder too long for \"ereg\\\" in its 0x100 bytes").into(),
        );
    }
    if switch(2) != 1
        && switch(5) != 1
        && switch(3) == 1
        && rd(v.from_cd) == 1
        && rd(v.command + 4) >= ROOM
    {
        return Err(BadInput("a command line's program of 0x100 bytes or more").into());
    }
    let mut g = *r;
    if switch(0) == 1 {
        part(ADD_ITEMS, &mut g)?;
    }
    if switch(1) == 1 {
        part(REMOVE_ITEMS, &mut g)?;
    }
    if switch(2) == 1 {
        for dir in [v.game_dir, v.other_dir] {
            g.edx = dir;
            if !checked(SET_CD, &mut g, part)? {
                return Ok(());
            }
        }
        g.eax = text(DXSETUP);
        g.ebx = 1;
        return part(RUN, &mut g);
    }
    if switch(5) == 1 {
        unsafe {
            g.ebx = v.game_dir;
            g.ecx = game_dir_len;
            g.ebx += g.ecx;
            g.edx = text(EREG);
            let (cursor, _) = append_directory(g.edx as *const u8, g.ebx as *mut u8)?;
            cursor.write(0);
            g.ebx = v.program;
            let cursor = append_file_name(g.edx as *const u8, g.ebx as *mut u8)?;
            cursor.write(0);
            g.ebx = cursor as u32 + 1;
        }
        g.edx = v.game_dir;
        if !checked(SET_CD, &mut g, part)? {
            return Ok(());
        }
        g.eax = v.program;
        g.ebx = 1;
        return part(RUN, &mut g);
    }
    if switch(3) != 1 {
        return Ok(());
    }
    g.esi = v.source;
    g.edx = v.installed;
    if !checked(NORMALISE, &mut g, part)? || !checked(SET_CD, &mut g, part)? {
        return Ok(());
    }
    if rd(v.from_cd) == 1 {
        g.ecx = rd(v.command + 4);
        g.esi = rd(v.command);
        g.edi = v.program;
        g.eax = g.edi;
        unsafe {
            std::ptr::copy(g.esi as *const u8, g.edi as *mut u8, g.ecx as usize);
            ((g.edi + g.ecx) as *mut u8).write(0);
        }
        g.esi += g.ecx;
        g.edi += g.ecx;
        g.ecx = 0;
        g.ebx = text(RAL);
        part(COPY_FILE, &mut g)?;
        g.eax = 1000;
        part(SLEEP, &mut g)?;
    }
    g.eax = text(RAL);
    g.ebx = 1;
    part(RUN, &mut g)
}

/// The calls in the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use windows_sys::Win32::System::Environment::SetCurrentDirectoryA;
    unsafe {
        match place {
            ADD_ITEMS | REMOVE_ITEMS | RUN => call_game(r, game_addr(rva(place as usize))),
            SET_CD => {
                r.eax = SetCurrentDirectoryA(r.edx as *const u8) as u32;
                r.set_carry(r.eax == 0);
            }
            NORMALISE => {
                let written = crate::install::normalise_path::normalise_path(
                    r.esi as *const u8,
                    r.edx as *mut u8,
                )?;
                r.set_carry(!written);
            }
            COPY_FILE => {
                let done =
                    crate::files::copy_file::copy_file(r.eax as *const u8, r.ebx as *const u8)?;
                r.set_carry(!done);
            }
            SLEEP => crate::system::sleep::sleep(r.eax),
            _ => return Err(BadInput("a call after the exit with no version").into()),
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if crate::framework::shadow() {
        unsafe { call_original(r) };
        return;
    }
    let mut g = *r;
    if let Err(e) = run_after_exit_with(&mut g, &Vars::game(), &mut game_part) {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // reported (4), switches (0x18), game dir and installed (0x200), other dir (4),
    // program (0x100), source (0x100), from CD (4), command (8)
    let mut mem = vec![0u8; 0x600];
    let base = mem.as_mut_ptr() as u32;
    let v = Vars {
        reported: base,
        switches: base + 4,
        game_dir: base + 0x20,
        installed: base + 0x120,
        other_dir: base + 0x220,
        program: base + 0x230,
        source: base + 0x330,
        from_cd: base + 0x430,
        command: base + 0x434,
    };
    let command = b"d:\\setup\\ral.exe";
    // pusha; cmp g_fatalReported, 1
    let entry = [0x60, 0x80, 0x3D, 0xBC, 0x67, 0x4F, 0x00, 0x01];
    let outs = [
        0x17, 0x25, 0x39, 0x4A, 0x60, 0x79, 0x86, 0x8B, 0x96, 0x9B, 0xA6, 0xB8, 0xD4, 0xDB, 0x10A,
        0x114, 0x124,
    ];
    let copy = match crate::system::api_stand_ins::copy_with(
        0x412FA2,
        0x12B,
        &entry,
        &outs,
        &[
            ADD_ITEMS,
            REMOVE_ITEMS,
            SET_CD,
            RUN,
            NORMALISE,
            COPY_FILE,
            SLEEP,
        ],
        &[],
        &[
            (REPORTED, 1, v.reported),
            (SWITCHES, 0x18, v.switches),
            (GAME_DIR, 0x200, v.game_dir),
            (OTHER_DIR, 2, v.other_dir),
            (PROGRAM, 0x100, v.program),
            (SOURCE, 0x100, v.source),
            (FROM_CD, 4, v.from_cd),
            (COMMAND, 8, v.command),
        ],
        18,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(base, 0x600usize)];
    for case in 0..48u32 {
        let switches = [
            case & 1,
            (case >> 1) & 1,
            (case % 5 == 2) as u32,
            (case >> 2) & 1,
            0,
            (case % 7 == 3) as u32,
        ];
        let reported = (case % 11 == 10) as u8;
        let from_cd = (case >> 3) & 1;
        let fill = || unsafe {
            let m = base as *mut u8;
            for k in 0..0x600 {
                m.add(k).write_volatile(0xAA);
            }
            m.write_volatile(reported);
            for (n, s) in switches.iter().enumerate() {
                ((v.switches + 4 * n as u32) as *mut u32).write_volatile(*s);
            }
            for (k, &b) in b"c:\\games\\rally\\\0".iter().enumerate() {
                ((v.game_dir + k as u32) as *mut u8).write_volatile(b);
            }
            (v.from_cd as *mut u32).write_volatile(from_cd);
            (v.command as *mut u32).write_volatile(command.as_ptr() as u32);
            ((v.command + 4) as *mut u32).write_volatile(command.len() as u32);
        };
        compare(
            &mut t,
            &format!("switches {switches:?}, reported {reported}, from the CD {from_cd}"),
            &copy,
            Regs::SENTINEL,
            &fill,
            &all,
            &|r, part| run_after_exit_with(r, &v, part),
            false,
        );
    }
    let _ = &mut mem;
    t.finish()
}
