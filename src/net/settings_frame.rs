//! `net::settings_frame` (0x4169E2): the frame of the network settings screen
//! (`net::settings_loop`): on the locked surface, the background, four settings from y 0x92
//! (`net::setting_label` and the value): the connection (string 0x163, `unk_52ACA4`), the
//! player's name (0x164, `g_machineName`), the most players (0x165, `dword_52ABD0`,
//! `net::draw_number_value`) and the host's address (0x166, `unk_52ABE4`); then the last
//! item's frame, right to left from 0x311 at y 0x21D (sprites 0x13, 0x12 once per 32 pixels
//! of string 0x16F's width, 0x11) and its string (`net::draw_centred_item`); unlocked, shown.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! The width's tiles are counted down with `dec` / `jnz`: a string of width 0 would go round
//! 2^32 times (refused). Called by `screens::run_frame` for `net::settings_loop`. Its calls
//! through `part`. Shadow mode runs the original alone (it draws). Self-test: a copy whose
//! calls go to stand-ins, against the Rust version: the lock failing, widths of 1, 0x20 and
//! 0x45.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    BLIT, COPY_PICTURE, HOST_ADDRESS, LOCK, MACHINE_NAME, MEASURE_STRING, MOST_PLAYERS, PALETTE,
    Part, SAVED_NAME, SHOW_FRAME, SPRITES, UNLOCK, a, answer, dispatch_lobby, g, run_both,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::settings_frame", rva(0x4169E2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LABEL: u32 = 0x416AF8; // net::setting_label
const CENTRED: u32 = 0x416B53; // net::draw_centred_item
const NUMBER: u32 = 0x416B8E; // net::draw_number_value
const VALUE: u32 = 0x416BC4; // net::draw_setting_value

/// The Rust version: the registers it leaves (the entry's).
pub fn settings_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
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
    part(COPY_PICTURE, &mut r)?;
    r.esi = 0;
    r.ebp = 0x92;
    for (label, value, how) in [
        (0x163, a(SAVED_NAME), VALUE),
        (0x164, a(MACHINE_NAME), VALUE),
        (0x165, g(MOST_PLAYERS), NUMBER),
        (0x166, a(HOST_ADDRESS), VALUE),
    ] {
        r.eax = label;
        part(LABEL, &mut r)?;
        r.eax = value;
        part(how, &mut r)?;
        r.esi = r.esi.wrapping_add(1);
    }
    r.eax = 0x16F;
    r.ebx = a(PALETTE);
    part(MEASURE_STRING, &mut r)?;
    let width = r.ecx;
    r.ecx = r.ecx.wrapping_add(0x1F) >> 5;
    let tiles = r.ecx;
    if tiles == 0 {
        return Err(BadInput("a last item of no width: the frame's loop never ends").into());
    }
    let blit = |r: &mut Regs, glyph: u32, step: u32, part: Part| -> Result<(), Refused> {
        let item = r.esi;
        let count = r.ecx;
        r.eax = glyph;
        r.ecx = r.edi.wrapping_sub(step);
        r.edi = r.ecx;
        r.edx = 0x21D;
        r.esi = g(SPRITES);
        part(BLIT, r)?;
        r.esi = item;
        r.ecx = count;
        Ok(())
    };
    r.edi = 0x311;
    blit(&mut r, 0x13, 0x22, part)?;
    r.ecx = tiles;
    loop {
        blit(&mut r, 0x12, 0x20, part)?;
        r.ecx = r.ecx.wrapping_sub(1);
        if r.ecx == 0 {
            break;
        }
    }
    let count = r.ecx;
    blit(&mut r, 0x11, 0x22, part)?;
    r.ecx = count;
    r.eax = 0x16F;
    r.ebp = 0x21D;
    r.ecx = tiles;
    r.ebx = width;
    r.ecx = (r.ecx << 5).wrapping_add(0x44).wrapping_sub(r.ebx);
    r.ecx = ((r.ecx as i32) >> 1) as u32;
    r.ecx = r.ecx.wrapping_neg().wrapping_add(0x311);
    part(CENTRED, &mut r)?;
    part(UNLOCK, &mut r)?;
    part(SHOW_FRAME, &mut r)?;
    done(&r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, settings_frame);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let places = [
        LOCK,
        COPY_PICTURE,
        LABEL,
        VALUE,
        NUMBER,
        MEASURE_STRING,
        BLIT,
        CENTRED,
        UNLOCK,
        SHOW_FRAME,
    ];
    // pusha; call gfx::lock_surface
    let copy = match crate::net::lobby::copy(
        0x4169E2,
        0x116,
        &[0x60, 0xE8, 0xC5, 0xEE, 0xFE, 0xFF],
        &[
            0x1, 0xC, 0x1D, 0x28, 0x33, 0x3E, 0x49, 0x53, 0x5E, 0x69, 0x7A, 0xA3, 0xC1, 0xE1,
            0x105, 0x10A, 0x10F,
        ],
        &places,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    for (fails, width) in [(true, 0x20u32), (false, 1), (false, 0x20), (false, 0x45)] {
        run_both(
            &mut t,
            &format!("the lock failing {fails}, width {width:#x}"),
            copy.addr(),
            Regs::SENTINEL,
            &[],
            0,
            &[],
            &|| {
                answer(Box::new(move |place, r| match place {
                    LOCK => r.set_carry(fails),
                    MEASURE_STRING => (r.eax, r.ebx, r.ecx, r.edx) = (0, 1, width, 0x30),
                    LABEL => {
                        r.ebx = 0;
                        r.ecx = 0x190;
                        r.edx = r.ebp;
                        r.ebp += 0x20;
                    }
                    VALUE | NUMBER => {
                        r.eax = 0;
                        r.ebp += 0x1E;
                    }
                    BLIT => (r.eax, r.ebx, r.edx) = (r.ecx, r.eax ^ 0x55, 0x99),
                    _ => {}
                }));
            },
            &settings_frame,
        );
    }
    t.finish()
}
