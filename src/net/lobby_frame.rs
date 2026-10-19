//! `net::lobby_frame` (0x417D95): the frame of the lobby (`net::lobby_loop`): on the locked
//! surface the background; the table's headings (strings 0x15F at 0x21A and 0x160 at 0x2C0,
//! `stru_51E564`, y 0x6F); the players sorted (`net::sort_players`) and one row each from y
//! 0x8C: the nationality's flag (sprite +0x27 + 0x1C at 0x18), the name (`stru_51E784` when
//! ready, else `stru_51E6C4`), the car's sprite (+0x25 + 0x44, 0x18 and 0x19 swapped), the
//! race's time and the total (`text::draw_time_text` with the hours always shown, or string
//! 0x161 for no time; with gaps shown, `dword_52AC30` 1, each as its gap to the first row's
//! with a '+' over its first character when within 0x36EE7F); the chat's eight lines (under
//! the received messages' lock) and its input (`net::chat_input_line`, item 3); the four
//! buttons right to left from 0x310 at 0x223 (`net::lobby_button`: ready 0x15D, the car 0x15B
//! and the stage 0x15C when they can be chosen, the chat 0x15E); two lines (`gfx::draw_hline`
//! at y 0x156 and 0x217, 0x320 long, colour 7); unlocked and shown.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! More than eight players would read past the sorted copy: refused. Called by
//! `screens::run_frame` for `net::lobby_loop`. Its calls through `part`
//! (`strings::append_nul` inline). Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose calls go to stand-ins (the texts written), against the Rust
//! version: the lock failing; no players; three players with and without gaps, ready or
//! not, times and none; the buttons' switches.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    APPEND_STRING, APPEND_TIME, BLIT, BLOCK, CAR_ITEM, CHAT, CHAT_INPUT, COPY_PICTURE, DRAW_HLINE,
    DRAW_STRING, DRAW_TEXT, DRAW_TIME, FONT, LOCK, LOCK_RECEIVED, PLAYERS, Part, SHOW_FRAME,
    SHOW_GAPS, SORTED, SPRITES, STAGE_ITEM, STR_APPEND, Saved, TEXT, UNLOCK, UNLOCK_RECEIVED, a,
    answer, dispatch_lobby, dword, g, p, region, run_both,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::lobby_frame", rva(0x417D95), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const HEADING_FONT: usize = 0x51E564; // stru_51E564
const NAME_FONT: usize = 0x51E6C4; // stru_51E6C4
const READY_FONT: usize = 0x51E784; // stru_51E784
const CHAT_FONT: usize = 0x51E584; // stru_51E584
const SHOW_HOURS: usize = 0x4E0148; // g_timeShowHours
const FONT_PALETTES: usize = 0x51DEB0; // g_fontPalettes
const SORT_PLAYERS: u32 = 0x418108; // net::sort_players
const CHAT_INPUT_LINE: u32 = 0x418083; // net::chat_input_line
const LOBBY_BUTTON: u32 = 0x41802F; // net::lobby_button
const MOST_GAP: u32 = 0x36_EE7F;

fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

fn byte(at: u32) -> u32 {
    unsafe { (at as *const u8).read() as u32 }
}

fn nul(r: &mut Regs) {
    unsafe { (r.ebx as *mut u8).write(0) };
    r.ebx = r.ebx.wrapping_add(1);
}

/// A time, or its gap to `first` with a '+' over its first character, into the text buffer:
/// eax the time; the registers as the original leaves them before its `append_nul`.
fn time_text(r: &mut Regs, gaps: bool, first: u32, part: Part) -> Result<(), Refused> {
    if gaps && r.eax != first {
        r.eax = r.eax.wrapping_sub(first);
        if r.eax <= MOST_GAP {
            r.ecx = r.ebx;
            part(APPEND_TIME, r)?;
            unsafe { (r.ecx as *mut u8).write(b'+') };
            return Ok(());
        }
    }
    part(APPEND_TIME, r)
}

/// The Rust version: the registers it leaves (the entry's).
pub fn lobby_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let done = |r: &Regs| {
        Ok(Regs {
            eflags: r.eflags,
            ..*entry
        })
    };
    part(LOCK, &mut r)?;
    if r.carry() {
        return done(&r);
    }
    if g(PLAYERS) > 8 {
        return Err(BadInput("more than eight players").into());
    }
    part(COPY_PICTURE, &mut r)?;
    r.ebp = 0x6F;
    for (id, x) in [(0x15F, 0x21A), (0x160, 0x2C0)] {
        r.eax = id;
        r.ebx = a(HEADING_FONT);
        r.ecx = x;
        r.edx = r.ebp;
        part(DRAW_STRING, &mut r)?;
    }
    r.ebp += 0x1D;
    if g(PLAYERS) != 0 {
        part(SORT_PLAYERS, &mut r)?;
        let gaps = g(SHOW_GAPS) == 1;
        let (first_time, first_total) = (get(a(SORTED) + 0x2C), get(a(SORTED) + 0x30));
        for n in 0..g(PLAYERS) {
            r.eax = n;
            let block = a(SORTED).wrapping_add(n.wrapping_mul(BLOCK));
            r.esi = block;
            r.edi = 0x18;
            let blit = |r: &mut Regs, glyph: u32, y: u32, part: Part| -> Result<(), Refused> {
                r.eax = glyph;
                r.ecx = r.edi;
                r.edx = y;
                r.esi = g(SPRITES);
                part(BLIT, r)?;
                r.esi = block;
                Ok(())
            };
            let y = r.ebp;
            blit(&mut r, byte(block + 0x27) + 0x1C, y, part)?;
            r.edi += 0x28;
            r.eax = block;
            r.ebx = a(TEXT);
            part(STR_APPEND, &mut r)?;
            nul(&mut r);
            r.eax = if byte(block + 0x24) & 1 != 0 {
                a(READY_FONT)
            } else {
                a(NAME_FONT)
            };
            r.ebx = a(TEXT);
            r.ecx = r.edi;
            r.edx = r.ebp;
            part(DRAW_TEXT, &mut r)?;
            r.edi += 0x14A;
            let mut car = byte(block + 0x25);
            if (0x18..=0x19).contains(&car) {
                car ^= 1;
            }
            let y = r.ebp + 1;
            blit(&mut r, car + 0x44, y, part)?;
            r.edi += 0x48;
            let kept_hours = g(SHOW_HOURS);
            p(SHOW_HOURS, 1);
            r.eax = get(block + 0x2C);
            r.ebx = a(TEXT);
            if r.eax == u32::MAX {
                r.eax = 0x161;
                part(APPEND_STRING, &mut r)?;
                nul(&mut r);
                r.eax = a(NAME_FONT);
                r.ebx = a(TEXT);
                r.ecx = r.edi;
                r.edx = r.ebp;
                part(DRAW_TEXT, &mut r)?;
            } else {
                time_text(&mut r, gaps, first_time, part)?;
                nul(&mut r);
                r.eax = a(FONT);
                r.ebx = a(TEXT);
                r.ecx = r.edi;
                r.edx = r.ebp;
                part(DRAW_TIME, &mut r)?;
            }
            r.edi += 0xA5;
            r.eax = get(block + 0x30);
            r.ebx = a(TEXT);
            let timed = get(block + 0x2C) != u32::MAX;
            time_text(&mut r, gaps && timed, first_total, part)?;
            nul(&mut r);
            r.eax = a(FONT);
            r.ebx = a(TEXT);
            r.ecx = r.edi;
            part(DRAW_TIME, &mut r)?;
            p(SHOW_HOURS, kept_hours);
            r.ebp = r.ebp.wrapping_add(get(a(NAME_FONT) + 8));
            r.eax = n + 1;
        }
    }
    r.ebp = 0x158;
    part(LOCK_RECEIVED, &mut r)?;
    r.eax = a(CHAT_FONT);
    r.ebx = a(CHAT);
    r.ecx = 0xA;
    r.edx = r.ebp;
    part(DRAW_TEXT, &mut r)?;
    part(UNLOCK_RECEIVED, &mut r)?;
    r.eax = get(a(CHAT_FONT) + 8).wrapping_mul(8);
    r.ebp = r.ebp.wrapping_add(r.eax);
    r.esi = 3;
    r.eax = a(CHAT_INPUT);
    part(CHAT_INPUT_LINE, &mut r)?;
    r.ebp = 0x223;
    r.edi = 0x310;
    r.esi = 0;
    for (id, shown) in [
        (0x15D, true),
        (0x15B, g(CAR_ITEM) == 1),
        (0x15C, g(STAGE_ITEM) == 1),
        (0x15E, true),
    ] {
        if shown {
            r.eax = id;
            part(LOBBY_BUTTON, &mut r)?;
        }
        if id != 0x15E {
            r.esi += 1;
        }
    }
    r.eax = 7;
    r.ebx = 0x320;
    r.ecx = 0;
    r.edx = 0x156;
    r.esi = g(FONT_PALETTES);
    part(DRAW_HLINE, &mut r)?;
    r.edx = 0x217;
    part(DRAW_HLINE, &mut r)?;
    part(UNLOCK, &mut r)?;
    part(SHOW_FRAME, &mut r)?;
    done(&r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, lobby_frame);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let places = [
        LOCK,
        COPY_PICTURE,
        DRAW_STRING,
        SORT_PLAYERS,
        BLIT,
        STR_APPEND,
        DRAW_TEXT,
        APPEND_STRING,
        APPEND_TIME,
        DRAW_TIME,
        LOCK_RECEIVED,
        UNLOCK_RECEIVED,
        CHAT_INPUT_LINE,
        LOBBY_BUTTON,
        DRAW_HLINE,
        UNLOCK,
        SHOW_FRAME,
    ];
    // pusha; call gfx::lock_surface
    let copy = match crate::net::lobby::copy(
        0x417D95,
        0x29A,
        &[0x60, 0xE8, 0x12, 0xDB, 0xFE, 0xFF],
        &[
            0x1, 0xC, 0x28, 0x3F, 0x54, 0x7F, 0x91, 0x96, 0xB2, 0xDD, 0x10A, 0x10F, 0x11F, 0x146,
            0x150, 0x155, 0x165, 0x1A0, 0x1AA, 0x1AF, 0x1BD, 0x1E1, 0x1F9, 0x1FE, 0x217, 0x22D,
            0x241, 0x255, 0x260, 0x27F, 0x289, 0x28E, 0x293,
        ],
        &places,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [
        dword(PLAYERS),
        dword(SHOW_GAPS),
        dword(CAR_ITEM),
        dword(STAGE_ITEM),
        dword(SHOW_HOURS),
        region(TEXT, 0x40),
        region(SORTED, 3 * BLOCK as usize),
    ];
    let saved = Saved::new(&all);
    // (what, the lock failing, players, gaps, car, stage)
    let cases: [(&str, bool, u32, u32, u32, u32); 6] = [
        ("the lock failing", true, 3, 0, 1, 1),
        ("no players", false, 0, 0, 1, 1),
        ("three", false, 3, 0, 1, 0),
        ("three with gaps", false, 3, 1, 0, 1),
        ("one with gaps", false, 1, 1, 0, 0),
        ("three, gaps 2", false, 3, 2, 1, 1),
    ];
    // (state, nationality, car, time, total) of the sorted rows
    let rows: [(u8, u8, u8, u32, u32); 3] = [
        (1, 3, 0x18, 6000, 9000),
        (0, 5, 0x19, u32::MAX, 0x40_0000),
        (1, 1, 0x05, 6500, 0x0100_0000),
    ];
    for (what, fails, players, gaps, car, stage) in cases {
        run_both(
            &mut t,
            what,
            copy.addr(),
            Regs::SENTINEL,
            &all,
            0,
            &[],
            &|| {
                p(PLAYERS, players);
                p(SHOW_GAPS, gaps);
                p(CAR_ITEM, car);
                p(STAGE_ITEM, stage);
                p(SHOW_HOURS, 0);
                for (n, &(state, nation, car, time, total)) in rows.iter().enumerate() {
                    let b = a(SORTED) + n as u32 * BLOCK;
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            c"Name".as_ptr() as *const u8,
                            b as *mut u8,
                            5,
                        );
                        ((b + 0x24) as *mut u8).write(state);
                        ((b + 0x25) as *mut u8).write(car);
                        ((b + 0x27) as *mut u8).write(nation);
                        ((b + 0x2C) as *mut u32).write_unaligned(time);
                        ((b + 0x30) as *mut u32).write_unaligned(total);
                    }
                }
                let mut inner = crate::net::draw_setting_value::string_answers();
                answer(Box::new(move |place, r| match place {
                    LOCK => r.set_carry(fails),
                    APPEND_TIME => {
                        let text = format!("{:08}", r.eax % 100_000_000);
                        unsafe {
                            std::ptr::copy_nonoverlapping(text.as_ptr(), r.ebx as *mut u8, 8)
                        };
                        r.ebx += 8;
                    }
                    BLIT => (r.eax, r.ebx, r.ecx, r.edx) = (1, 2, 3, 4),
                    LOBBY_BUTTON => r.edi -= 0x60,
                    _ => inner(place, r),
                }));
            },
            &lobby_frame,
        );
    }
    drop(saved);
    t.finish()
}
