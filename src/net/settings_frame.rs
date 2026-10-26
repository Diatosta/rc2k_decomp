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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{
    BLIT, COPY_PICTURE, HOST_ADDRESS, LOCK, MACHINE_NAME, MEASURE_STRING, MOST_PLAYERS, PALETTE,
    Part, SAVED_NAME, SHOW_FRAME, SPRITES, UNLOCK, a, g, lobby_entry,
};
use rc2k_core::net::screens;

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
    // Converted: `rc2k_core::net::screens::last_item` (and the frame's sprites).
    let (tiles, text_x) = screens::last_item(width)?;
    r.ecx = tiles;
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
    r.edi = screens::LAST_ITEM_RIGHT;
    blit(&mut r, screens::RIGHT_END, screens::END_WIDTH, part)?;
    r.ecx = tiles;
    loop {
        blit(&mut r, screens::TILE, screens::TILE_WIDTH, part)?;
        r.ecx = r.ecx.wrapping_sub(1);
        if r.ecx == 0 {
            break;
        }
    }
    blit(&mut r, screens::LEFT_END, screens::END_WIDTH, part)?;
    r.eax = 0x16F;
    r.ebp = 0x21D;
    r.ebx = width;
    r.ecx = text_x;
    part(CENTRED, &mut r)?;
    part(UNLOCK, &mut r)?;
    part(SHOW_FRAME, &mut r)?;
    done(&r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, settings_frame)
}
