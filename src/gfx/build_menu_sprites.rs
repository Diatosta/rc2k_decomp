//! `gfx::build_menu_sprites` (0x47C4A1): builds the menus' sprite table.
//!
//! In:  nothing
//! Out: every register preserved
//! `begin_glyph_tables`, the builder's flags = 1 (run-length coded), a new table on the
//! fonts' list `stru_51B984`, `build_glyphs` on `files\g\graphics\sprites\mbuts00.pcx`
//! (that sheet and `mbuts01.pcx`), the table's address to `g_spriteTable` (0x755581),
//! `end_glyph_tables`. The list is not rewound: every call takes new memory after what
//! is there. Callers: `sub_47C63F` (the main menu's start), `sub_4154CD`, `sub_4703C2`.
//! Shadow mode runs the original alone (the list's memory can be taken once) and checks
//! that the sprite table is the table built; the building itself is compared in
//! `gfx::build_glyphs`' hook. Self-test: a redirected copy whose four calls note
//! themselves and which stores the table's address in a test variable.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::gfx::build_glyphs::{BUILDER_FLAGS_RVA, TABLE_RVA};
use crate::gfx::glyph_builder::RUN_LENGTHS;
use crate::gfx::glyph_tables::{
    BEGIN_TABLE, BEGIN_TABLES, BUILD_GLYPHS, END_TABLES, Game, Noted, PLACES, Parts,
    TABLES_LIST_RVA, stand_in_for,
};
use crate::menu::frame_test;
use crate::redirect::RedirectedCopy;
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::build_menu_sprites", rva(0x47C4A1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const SPRITE_TABLE_RVA: usize = rva(0x755581); // g_spriteTable
const SHEETS_RVA: usize = rva(0x732C93); // "files\g\graphics\sprites\mbuts00.pcx"

// ---- Rust version ----

/// `entry` are the registers at entry; the table's address goes to `sprite_table`.
pub unsafe fn build_menu_sprites(
    entry: &Regs,
    parts: &mut dyn Parts,
    sprite_table: *mut u32,
) -> Result<(), Refused> {
    unsafe {
        let mut g = *entry;
        g.ebx = game_addr(TABLES_LIST_RVA) as u32;
        parts.call(BEGIN_TABLES, &mut g)?;
        game_write(BUILDER_FLAGS_RVA, RUN_LENGTHS);
        parts.call(BEGIN_TABLE, &mut g)?;
        g.eax = game_addr(SHEETS_RVA) as u32;
        parts.call(BUILD_GLYPHS, &mut g)?;
        g.eax = game_read(TABLE_RVA);
        sprite_table.write_unaligned(g.eax);
        parts.call(END_TABLES, &mut g)?;
        Ok(())
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let sprite_table = game_addr(SPRITE_TABLE_RVA) as *mut u32;
    if !shadow() {
        let entry = *r;
        if let Err(e) = unsafe { build_menu_sprites(&entry, &mut Game, sprite_table) } {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let (sprites, table): (u32, u32) =
        unsafe { (game_read(SPRITE_TABLE_RVA), game_read(TABLE_RVA)) };
    let flags: u32 = unsafe { game_read(BUILDER_FLAGS_RVA) };
    if sprites != table || flags & RUN_LENGTHS == 0 || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| {
            format!("the original left the sprite table {sprites:#x}, the table built {table:#x}, flags {flags:#x}")
        });
    }
}

// ---- self-test ----

const LEN: usize = 0x37;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x8D, 0x1D, 0x84, 0xB9, 0x51, 0x00, 0xE8];
const CALLS: [usize; 4] = [0x07, 0x16, 0x21, 0x30];
/// Where the copy names `g_spriteTable`.
const SPRITE_TABLE_OPERAND: usize = 0x2C;

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut sprite_table = 0u32;
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x47C4A1)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &stand_in_for,
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    unsafe { copy.patch_u32(SPRITE_TABLE_OPERAND, &mut sprite_table as *mut u32 as u32) };
    let variables = [at(BUILDER_FLAGS_RVA)];
    let saved = capture(&variables);
    for flags in [0u32, 0x33] {
        let before = Regs::SENTINEL;
        let mut run = |rust: bool| unsafe {
            game_write(BUILDER_FLAGS_RVA, flags);
            sprite_table = 0xAAAA_AAAA;
            frame_test::begin(u32::MAX, 0);
            let preserved = if rust {
                build_menu_sprites(&before, &mut Noted(&PLACES), &mut sprite_table).is_ok()
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                ro.preserved_except(&before, 0)
            };
            (
                preserved,
                frame_test::notes(),
                sprite_table,
                game_read::<u32>(BUILDER_FLAGS_RVA),
            )
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 && theirs.1.len() == 4, || {
            format!("flags={flags:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&variables, &saved);
    t.finish()
}
