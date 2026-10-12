//! `gfx::show_picture_file` (0x41172B): loads a picture and shows it at once, on every
//! page of the screen: the loading picture.
//!
//! In:  eax = the picture's path
//! Out: eax = what `dword_51B9C4` held (1, unless the file could not be read: then the
//!      path); the rest as its callees leave them
//! The window's messages are passed on (`sub_410535`); `dword_51B9C4` = 1; the picture's
//! size is set to 800 x 600 with row length 0xFFFFFFFD and the file is read
//! (`gfx::read_picture_file`); the size then becomes the screen's, the row length the
//! screen's width * 4.
//! - Not read: `gfx::clear_picture`, and on as for a picture of the screen's width.
//! - Read: `dword_51B9C4` is taken (set to 0); if it was not 0 and the fonts are built
//!   (`g_fontPalettes`), the picture's flags become 0x802, the picture is made the
//!   surface (`gfx::picture_as_surface`) and string 0xC0 is drawn on it at (790, 47) with
//!   the font at 0x51EAA4.
//! - A picture of the screen's width: `gfx::convert_picture`, then twice
//!   `gfx::copy_picture_to_surface` and `gfx::show_frame`, and a third time if the
//!   screen mode has bit 0x40000 (three pages).
//! - Any other width: as many times, `gfx::lock_surface` (failing ends it),
//!   `gfx::rescale_to_surface`, `gfx::unlock_surface`, `gfx::show_frame`.
//!
//! One caller, `menu::show_loading_picture`.
//! Every call goes through the game's address, so each callee's own hook decides which
//! version of it runs; the registers are handed from one to the next as they come back.
//! Shadow mode runs the original alone (it shows frames). Manual patch with a private
//! copy (0x138 bytes, 24 calls relocated): the function begins with a call.
//! Self-test: a redirected copy whose eleven kinds of call go to the test.

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("gfx::show_picture_file", rva(0x41172B), thunk, &ORIGINAL)
    .private_copy(LEN)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const SHOWING_RVA: usize = rva(0x51B9C4); // dword_51B9C4
const PICTURE_FLAGS_RVA: usize = rva(0x4F49CC); // g_pictureFlags
const PICTURE_WIDTH_RVA: usize = rva(0x4F49D0); // g_pictureWidth, height, row bytes
const SCREEN_WIDTH_RVA: usize = rva(0x4F494C); // g_screenWidth
const SCREEN_HEIGHT_RVA: usize = rva(0x4F4950); // g_screenHeight
const SCREEN_MODE_FLAGS_RVA: usize = rva(0x4F4934); // g_screenModeFlags
const THREE_PAGES: u32 = 0x40000;
const DECODED_WIDTH_RVA: usize = rva(0x519124); // g_decodedWidth
const FONT_PALETTES_RVA: usize = rva(0x51DEB0); // g_fontPalettes
const LOADING_FONT_RVA: usize = rva(0x51EAA4); // stru_51EAA4

/// The callees, in the order of the test's stand-ins.
const PUMP_MESSAGES: usize = 0x410535;
const READ_PICTURE_FILE: usize = 0x4119F4;
const PICTURE_AS_SURFACE: usize = 0x405FE7;
const DRAW_STRING_BY_ID: usize = 0x40234A;
const LOCK_SURFACE: usize = 0x4058AD;
const RESCALE_TO_SURFACE: usize = 0x4063B7;
const UNLOCK_SURFACE: usize = 0x405949;
const SHOW_FRAME: usize = 0x4057D1;
const CLEAR_PICTURE: usize = 0x4060D1;
const CONVERT_PICTURE: usize = 0x406126;
const COPY_PICTURE_TO_SURFACE: usize = 0x4060E3;

// ---- Rust version ----

/// Calls the function at that address (as IDA shows it) with the registers, which come
/// back as it leaves them.
pub type CallAt<'a> = &'a mut dyn FnMut(usize, &mut Regs);

/// `r` holds the registers at entry (eax the path) and at the end those to return with.
pub unsafe fn show_picture_file(r: &mut Regs, call: CallAt) {
    unsafe {
        call(PUMP_MESSAGES, r);
        game_write(SHOWING_RVA, 1u32);
        game_write(PICTURE_WIDTH_RVA, [800u32, 600, 0xFFFF_FFFD]);
        call(READ_PICTURE_FILE, r);
        let read = !r.carry();
        let (width, height): (u32, u32) =
            (game_read(SCREEN_WIDTH_RVA), game_read(SCREEN_HEIGHT_RVA));
        game_write(PICTURE_WIDTH_RVA, [width, height, width << 2]);
        let pages = if game_read::<u32>(SCREEN_MODE_FLAGS_RVA) & THREE_PAGES != 0 {
            3
        } else {
            2
        };
        if !read {
            call(CLEAR_PICTURE, r);
        } else {
            r.eax = game_read(SHOWING_RVA);
            game_write(SHOWING_RVA, 0u32);
            if r.eax != 0 && game_read::<u32>(FONT_PALETTES_RVA) != 0 {
                // Between `pusha` and `popa`: what these two leave is not kept.
                let mut inner = *r;
                game_write(PICTURE_FLAGS_RVA, 0x802u32);
                call(PICTURE_AS_SURFACE, &mut inner);
                inner.eax = 0xC0;
                inner.ebx = game_addr(LOADING_FONT_RVA) as u32;
                inner.ecx = 0x316;
                inner.edx = 0x2F;
                call(DRAW_STRING_BY_ID, &mut inner);
            }
            if game_read::<u32>(DECODED_WIDTH_RVA) != width {
                for _ in 0..pages {
                    call(LOCK_SURFACE, r);
                    if r.carry() {
                        return;
                    }
                    call(RESCALE_TO_SURFACE, r);
                    call(UNLOCK_SURFACE, r);
                    call(SHOW_FRAME, r);
                }
                return;
            }
        }
        call(CONVERT_PICTURE, r);
        for _ in 0..pages {
            call(COPY_PICTURE_TO_SURFACE, r);
            call(SHOW_FRAME, r);
        }
    }
}

// ---- hook ----

fn in_game(at: usize, r: &mut Regs) {
    unsafe { call_game(r, game_addr(rva(at))) };
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let esp = r.esp;
    unsafe { show_picture_file(r, &mut in_game) };
    r.esp = esp;
}

// ---- self-test ----

const LEN: usize = 0x138;
const ENTRY_BYTES: [u8; 8] = [0xE8, 0x05, 0xEE, 0xFF, 0xFF, 0xC7, 0x05, 0xC4];
const CALLS: [usize; 24] = [
    0x0, 0x2D, 0x85, 0x9F, 0xB4, 0xBB, 0xC0, 0xC5, 0xCA, 0xD1, 0xD6, 0xDB, 0xEC, 0xF3, 0xF8, 0xFD,
    0x103, 0x108, 0x10D, 0x112, 0x117, 0x11C, 0x12D, 0x132,
];
const CALLEES: [usize; 11] = [
    PUMP_MESSAGES,
    READ_PICTURE_FILE,
    PICTURE_AS_SURFACE,
    DRAW_STRING_BY_ID,
    LOCK_SURFACE,
    RESCALE_TO_SURFACE,
    UNLOCK_SURFACE,
    SHOW_FRAME,
    CLEAR_PICTURE,
    CONVERT_PICTURE,
    COPY_PICTURE_TO_SURFACE,
];

/// What a case's stand-ins do, and what was called.
struct Script {
    /// The file cannot be read.
    unread: bool,
    /// The decoded picture's width the reading leaves.
    decoded_width: u32,
    /// Which locking of the surface fails, from 1 (0: none).
    lock_fails: u32,
    locks: u32,
    /// Each call: its place, the picture's size and row length at it, and for the
    /// reading and the drawing the registers they are given.
    calls: Vec<[u32; 8]>,
}

static SCRIPT: Mutex<Script> = Mutex::new(Script {
    unread: false,
    decoded_width: 0,
    lock_fails: 0,
    locks: 0,
    calls: Vec::new(),
});

fn handler(place: u32, r: &mut Regs) {
    let mut script = SCRIPT.lock().unwrap();
    let size: [u32; 3] = unsafe { game_read(PICTURE_WIDTH_RVA) };
    let given = match place {
        1 => [r.eax, 0, 0, 0],
        3 => [r.eax, r.ebx, r.ecx, r.edx],
        _ => [0; 4],
    };
    script.calls.push([
        place, size[0], size[1], size[2], given[0], given[1], given[2], given[3],
    ]);
    match place {
        1 => {
            unsafe { game_write(DECODED_WIDTH_RVA, script.decoded_width) };
            r.set_carry(script.unread);
        }
        4 => {
            script.locks += 1;
            r.set_carry(script.locks == script.lock_fails);
        }
        _ => {}
    }
}

fn through_the_test(at: usize, r: &mut Regs) {
    let place = CALLEES.iter().position(|&known| known == at).unwrap() as u32;
    act(place, r);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x41172B)),
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
    let regions: [Region; 3] = [
        at(SHOWING_RVA),
        at(PICTURE_FLAGS_RVA),
        (game_addr(PICTURE_WIDTH_RVA) as u32, 12),
    ];
    let inputs = [
        at(DECODED_WIDTH_RVA),
        at(FONT_PALETTES_RVA),
        at(SCREEN_MODE_FLAGS_RVA),
    ];
    let saved = (capture(&regions), capture(&inputs));
    let screen: u32 = unsafe { game_read(SCREEN_WIDTH_RVA) };
    let mode: u32 = unsafe { game_read(SCREEN_MODE_FLAGS_RVA) };
    let path = c"a\\picture.png".as_ptr() as u32;
    // (what, unread, another width, the fonts are built, three pages, which lock fails)
    let cases: [(&str, bool, bool, bool, bool, u32); 10] = [
        (
            "the screen's width, two pages",
            false,
            false,
            true,
            false,
            0,
        ),
        (
            "the screen's width, three pages",
            false,
            false,
            true,
            true,
            0,
        ),
        ("no fonts yet", false, false, false, false, 0),
        ("not read", true, false, true, false, 0),
        ("not read, three pages", true, true, false, true, 0),
        ("another width, two pages", false, true, true, false, 0),
        ("another width, three pages", false, true, false, true, 0),
        ("the first lock fails", false, true, true, true, 1),
        ("the second lock fails", false, true, true, true, 2),
        ("the third lock fails", false, true, true, true, 3),
    ];
    for (what, unread, other_width, fonts, three_pages, lock_fails) in cases {
        let before = Regs {
            eax: path,
            ..Regs::SENTINEL
        };
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.0.len()]);
            game_write(DECODED_WIDTH_RVA, 0x5555u32);
            game_write(FONT_PALETTES_RVA, fonts as u32 * 0x1234);
            game_write(
                SCREEN_MODE_FLAGS_RVA,
                (mode & !THREE_PAGES) | if three_pages { THREE_PAGES } else { 0 },
            );
            *SCRIPT.lock().unwrap() = Script {
                unread,
                decoded_width: if other_width { screen + 7 } else { screen },
                lock_fails,
                locks: 0,
                calls: Vec::new(),
            };
            set_handler(handler);
            let mut ro = before;
            if rust {
                show_picture_file(&mut ro, &mut through_the_test);
            } else {
                call_game(&mut ro, copy.addr());
            }
            let ended = (ro.eax, ro.preserved_except(&before, crate::framework::EAX));
            (
                ended,
                capture(&regions),
                SCRIPT.lock().unwrap().calls.clone(),
            )
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0.1, || {
            format!(
                "{what}: original ended {:x?} with {} calls, left {:x?}; rust ended {:x?} with {} calls, left {:x?}; first different call {:?}",
                theirs.0,
                theirs.2.len(),
                theirs.1,
                mine.0,
                mine.2.len(),
                mine.1,
                theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b),
            )
        });
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
