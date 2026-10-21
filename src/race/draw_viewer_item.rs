//! `race::draw_viewer_item` (0x41D94A): one item of the replay viewer's list drawn, unless
//! hidden (bit 6): its picture (`[esi+0xC]`, a texture rectangle; for the three switch
//! items, `off_5C2C70` by the block's camera byte 2 for item kind 0xE or 3 otherwise,
//! `off_5C2C78` by byte 5, `off_5C2C88` by byte 4, the table's entry) as a quad
//! (`drive::tyre_quad`) on the item's place (`[esi+4]`: x, y, w, h) at the picture's
//! width (its absolute value), centred (`dword_5C29FC`); grey (0x666666) when marked
//! unusable (bit 7), else white, or for the highlighted item (ecx `dword_51D407`) red
//! while a camera is edited, else yellow and red by turns (bit 3 of `dword_51D41F -
//! dword_51D423`).
//!
//! In:  esi the item (16 bytes), ecx its number
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Caller: `race::viewer_frame` (each item). Its call through `part` (1 `tyre_quad`, eax
//! the picture, ebx the place, ecx the colour; in the game its Rust version). Shadow mode
//! runs the original alone (it draws). Self-test: a copy whose call goes to a stand-in,
//! against the Rust version: items hidden, unusable, highlighted (edited or not, both
//! turns), plain, and the three switch items with their bytes.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, g, p};
use crate::race::viewer::{
    BLOCK, CAMERA_EDIT, CAMERAS, CHOICE, FRAME_CLOCK, MOVE_CLOCK, Part, Saved, dispatch_parts,
    dword, gb, pb, region, run_both, test_part,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_viewer_item", rva(0x41D94A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SWITCH_A: usize = 0x5C2C70; // off_5C2C70: by camera byte 2 or 3
const SWITCH_B: usize = 0x5C2C78; // off_5C2C78: by byte 5
const SWITCH_C: usize = 0x5C2C88; // off_5C2C88: by byte 4
const PLACE: usize = 0x5C29FC; // dword_5C29FC: the quad's place

fn camera_byte(byte: usize) -> u32 {
    gb((CAMERAS + byte).wrapping_add(8 * g(BLOCK) as usize)) as u32
}

/// The Rust version: the registers it leaves (the entry's).
pub fn draw_viewer_item(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let item = entry.esi;
    let byte = |n: u32| unsafe { ((item + n) as *const u8).read() };
    let word = |at: u32| unsafe { (at as *const u32).read_unaligned() };
    if byte(0) & 0x40 != 0 {
        return Ok(*entry);
    }
    let mut picture = word(item + 0xC);
    // Converted: `rc2k_core::race::viewer::{switch_byte, centred, item_colour}`.
    use rc2k_core::race::viewer::{centred, item_colour, switch_byte};
    if picture == a(SWITCH_A) {
        let n = camera_byte(switch_byte(byte(1)) as usize);
        picture = word(picture.wrapping_add(4 * n));
    }
    if picture == a(SWITCH_B) {
        picture = word(picture.wrapping_add(4 * camera_byte(5)));
    }
    if picture == a(SWITCH_C) {
        picture = word(picture.wrapping_add(4 * camera_byte(4)));
    }
    let place = word(item + 4);
    let width = (word(picture + 8) as i32).unsigned_abs();
    p(PLACE + 8, width);
    p(PLACE, centred(word(place), word(place + 8), width));
    p(PLACE + 4, word(place + 4));
    p(PLACE + 0xC, word(place + 0xC));
    let colour = item_colour(
        byte(0),
        entry.ecx == g(CHOICE),
        || g(CAMERA_EDIT) != 0,
        || g(FRAME_CLOCK).wrapping_sub(g(MOVE_CLOCK)),
    );
    let mut r = Regs {
        eax: picture,
        ebx: a(PLACE),
        ecx: colour,
        edx: picture,
        edi: place,
        ..*entry
    };
    part(1, &mut r)?;
    Ok(*entry)
}

/// In the game: `tyre_quad`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    *r =
        unsafe { crate::drive::tyre_quad::tyre_quad(r, &mut crate::drive::tyre_parts::game_part) }?;
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        draw_viewer_item(r, &mut game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; mov edx, [esi+0Ch]; test byte ptr [esi], 40h
    let copy = match crate::race::viewer::copy(
        0x41D94A,
        0xDB,
        &[0x60, 0x8B, 0x56, 0x0C, 0xF6, 0x06, 0x40],
        &[0xD4],
        &[(0x41DD3B, 1)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::race::viewer::quiet();
    let regions = [region(PLACE, 0x10)];
    let mut all = regions.to_vec();
    all.extend([
        dword(BLOCK),
        dword(CHOICE),
        dword(CAMERA_EDIT),
        dword(FRAME_CLOCK),
        dword(MOVE_CLOCK),
        region(CAMERAS, 0x40),
    ]);
    let saved = Saved::new(&all);
    // A picture (x, y, w, h) of negative width, a place, and the item.
    let picture: [i32; 4] = [10, 20, -48, 16];
    let place: [u32; 4] = [100, 200, 64, 24];
    let mut item = [0u8; 16];
    for (what, flags, kind, number, pictured, edit, clock) in [
        ("hidden", 0x40u8, 0u8, 0u32, 0u32, 0u32, 0u32),
        ("unusable", 0x80, 0, 0, 0, 0, 0),
        ("plain", 0x00, 0, 1, 0, 0, 0),
        ("highlighted, edited", 0x00, 0, 3, 0, 1, 0),
        ("highlighted, yellow", 0x00, 0, 3, 0, 0, 8),
        ("highlighted, red", 0x00, 0, 3, 0, 0, 4),
        ("switch A, kind 0xE", 0x00, 0xE, 1, 1, 0, 0),
        ("switch A, kind 2", 0x00, 2, 1, 1, 0, 0),
        ("switch B", 0x00, 0, 1, 2, 0, 0),
        ("switch C", 0x00, 0, 1, 3, 0, 0),
    ] {
        item[0] = flags;
        item[1] = kind;
        let picture_at = match pictured {
            1 => a(SWITCH_A),
            2 => a(SWITCH_B),
            3 => a(SWITCH_C),
            _ => picture.as_ptr() as u32,
        };
        item[4..8].copy_from_slice(&(place.as_ptr() as u32).to_le_bytes());
        item[0xC..0x10].copy_from_slice(&picture_at.to_le_bytes());
        run_both(
            &mut t,
            what,
            copy.addr(),
            Regs {
                esi: item.as_ptr() as u32,
                ecx: number,
                ..Regs::SENTINEL
            },
            &regions,
            0,
            &|| {
                p(BLOCK, 2);
                p(CHOICE, 3);
                p(CAMERA_EDIT, edit);
                p(FRAME_CLOCK, 0x100 + clock);
                p(MOVE_CLOCK, 0x100);
                crate::race::viewer::fill(CAMERAS, 0x40, 24);
                for byte in 2..6 {
                    pb(CAMERAS + 16 + byte, (byte % 2) as u8);
                }
                crate::race::viewer::fill(PLACE, 0x10, 25);
            },
            &|r| draw_viewer_item(r, &mut test_part),
        );
    }
    drop(saved);
    t.finish()
}
