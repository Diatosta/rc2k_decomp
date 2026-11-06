//! The fonts' and the sprites' palettes and glyph tables, Rust's (phase 13, the font model,
//! 2026-10-27). The game carved them from heap blocks (`build_fonts`, `build_game_fonts`,
//! `build_glyphs`, `build_menu_sprites`, the installer's bar glyphs) and held their
//! addresses in the font objects (+0x14 palette, +0x18 glyph table), the sprite tables'
//! variables and the screens' palettes. Here each is bytes of its own, named by a place
//! below 0x10000, where no game address lies (`address_map`):
//! - a palette (0x400 bytes, 256 pixels): `id << 10 | offset`, ids 1 .. 15, so a place in
//!   it (the fonts' `palettes + 0x180`) is the palette's number plus the offset, as the
//!   address was;
//! - a glyph table: `0x4000 + id`, growing as sheets are appended.
//!
//! The lists keep their counts (`memory::owned::count_on` in place of `allocate`, the
//! cursor carved as before): running out is the original's fatal error at the same point.
//! What lies on a list is given back by its `free_list`. A place at or above 0x10000 is a
//! game address (the image's palettes, a screen's own): read and written there as before.

use crate::errors::BadInput;
use crate::gfx::owner::gfx;
use crate::pointers::{GameBytes, GamePtr};
use std::sync::{Arc, Mutex};

/// A palette's bytes.
pub const PALETTE_BYTES: usize = 0x400;
const PALETTE_IDS: usize = 16;
const TABLE_FIRST: u32 = 0x4000;
const PLACE_END: u32 = 0x1_0000;

struct Held {
    /// The list it is counted on.
    list: GamePtr,
    bytes: Arc<Vec<u8>>,
}

struct Store {
    palettes: [Option<Held>; PALETTE_IDS],
    tables: Vec<Option<Held>>,
}

static STORE: Mutex<Store> = Mutex::new(Store {
    palettes: [const { None }; PALETTE_IDS],
    tables: Vec::new(),
});

fn store() -> std::sync::MutexGuard<'static, Store> {
    STORE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Whether `v` names a palette or a table of the store (not a game address).
pub const fn is_place(v: u32) -> bool {
    v >= 0x400 && v < PLACE_END
}

/// The held bytes and the offset `place` names in them.
fn slot(s: &mut Store, place: u32) -> Option<(&mut Arc<Vec<u8>>, usize)> {
    if place >= TABLE_FIRST {
        let id = (place - TABLE_FIRST) as usize;
        let held = s.tables.get_mut(id)?.as_mut()?;
        Some((&mut held.bytes, 0))
    } else {
        let held = s.palettes.get_mut((place >> 10) as usize)?.as_mut()?;
        Some((&mut held.bytes, (place & 0x3FF) as usize))
    }
}

/// A new palette, zeroed, counted on nothing (the caller has carved its bytes off `list`):
/// its place. Refused when all fifteen are held.
pub fn new_palette(list: crate::pointers::GamePtr) -> Result<u32, BadInput> {
    let mut s = store();
    let id = (1..PALETTE_IDS)
        .find(|&id| s.palettes[id].is_none())
        .ok_or(BadInput("more than fifteen palettes held"))?;
    s.palettes[id] = Some(Held {
        list,
        bytes: Arc::new(vec![0; PALETTE_BYTES]),
    });
    Ok((id as u32) << 10)
}

/// A new glyph table, empty: its place. Refused past the places there are.
pub fn new_table(list: crate::pointers::GamePtr) -> Result<u32, BadInput> {
    let mut s = store();
    let held = Some(Held {
        list,
        bytes: Arc::new(Vec::new()),
    });
    let id = match s.tables.iter().position(Option::is_none) {
        Some(id) => {
            s.tables[id] = held;
            id
        }
        None => {
            s.tables.push(held);
            s.tables.len() - 1
        }
    };
    let place = TABLE_FIRST + id as u32;
    if place >= PLACE_END {
        s.tables[id] = None;
        return Err(BadInput("more glyph tables held than there are places"));
    }
    Ok(place)
}

/// Everything held on `list` given back (`memory::free_list`).
pub fn give_back(list: GamePtr) {
    let mut s = store();
    let Store { palettes, tables } = &mut *s;
    for slot in palettes.iter_mut().chain(tables.iter_mut()) {
        if slot.as_ref().is_some_and(|h| h.list == list) {
            *slot = None;
        }
    }
}

/// Where `place` is: when it is an IDA address of the jade font's block (the four menu
/// fonts name its glyphs and colour by the image's numbers, phase 13), a place in
/// `JADE_FONT`, bounded to it; else the game address itself (the image's palettes, a
/// screen's own: numbers the font objects keep). No game address lies in the jade range:
/// the image's range is reserved.
fn resolved(place: u32) -> GamePtr {
    let ida = place as usize;
    let jade = crate::gfx::jade_store::JadeFont::START;
    if (jade..jade + crate::gfx::jade_store::JadeFont::LEN).contains(&ida) {
        // The block starts 4 bytes into the store (`JadeFont::home`).
        GamePtr::to(&gfx().fonts.jade).offset((4 + ida - jade) as u32)
    } else {
        GamePtr::new(place)
    }
}

/// The bytes from a place or a game address on.
#[derive(Clone, Debug)]
pub enum Glyphs {
    Game(GameBytes),
    Held(Arc<Vec<u8>>, usize),
}

impl rc2k_core::strings::bytes::Bytes for Glyphs {
    fn byte(&self, at: usize) -> Option<u8> {
        match self {
            Glyphs::Game(b) => b.byte(at),
            Glyphs::Held(b, from) => b.get(from.checked_add(at)?).copied(),
        }
    }
    fn copy_run(&self, at: usize, out: &mut [u8]) -> Option<()> {
        match self {
            Glyphs::Game(b) => b.copy_run(at, out),
            Glyphs::Held(..) => {
                out.copy_from_slice(self.run(at, out.len())?);
                Some(())
            }
        }
    }
    fn run(&self, at: usize, len: usize) -> Option<&[u8]> {
        match self {
            Glyphs::Game(_) => None,
            Glyphs::Held(b, from) => {
                let start = from.checked_add(at)?;
                b.get(start..start.checked_add(len)?)
            }
        }
    }
}

/// The bytes from `place` on: a palette or table of the store, or game memory.
pub fn bytes(place: u32, what: &str) -> Result<Glyphs, BadInput> {
    if !is_place(place) {
        return Ok(Glyphs::Game(resolved(place).bytes(what)?));
    }
    let mut s = store();
    let (b, from) = slot(&mut s, place).ok_or(BadInput("no palette or glyph table there"))?;
    Ok(Glyphs::Held(Arc::clone(b), from))
}

/// `len` bytes from `place` + `at`.
pub fn read_bytes(place: u32, at: u32, len: usize, what: &str) -> Result<Vec<u8>, BadInput> {
    if !is_place(place) {
        return resolved(place).offset(at).read_bytes(len, what);
    }
    let mut s = store();
    let (b, from) = slot(&mut s, place).ok_or(BadInput("no palette or glyph table there"))?;
    let start = from + at as usize;
    b.get(start..start + len)
        .map(<[u8]>::to_vec)
        .ok_or(BadInput("a read past a palette or glyph table"))
}

/// `bytes` written at `place` + `at`; a table grows to take them, a palette refuses a
/// write past its end.
pub fn write_bytes(place: u32, at: u32, bytes: &[u8], what: &str) -> Result<(), BadInput> {
    if !is_place(place) {
        return resolved(place).offset(at).write_bytes(bytes, what);
    }
    let table = place >= TABLE_FIRST;
    let mut s = store();
    let (b, from) = slot(&mut s, place).ok_or(BadInput("no palette or glyph table there"))?;
    let start = from + at as usize;
    let end = start + bytes.len();
    let b = Arc::make_mut(b);
    if end > b.len() {
        if !table {
            return Err(BadInput("a write past a palette"));
        }
        b.resize(end, 0);
    }
    b[start..end].copy_from_slice(bytes);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc2k_core::strings::bytes::Bytes as _;

    #[test]
    fn the_jade_fonts_image_numbers_resolve_to_its_block() {
        let fonts = &gfx().fonts;
        use crate::gfx::jade_store::JadeFont;
        let _lock = crate::store::test_lock();
        // The glyph table and the palette (the colour), as the menu fonts name them.
        let table = bytes(0x4F52C4, "the jade glyphs").unwrap();
        let colour = read_bytes(0x4F66C4, 0, 4, "the jade colour").unwrap();
        assert_eq!(colour, fonts.jade.colour.get().to_le_bytes());
        assert_eq!(table.byte(0), Some(fonts.jade.glyphs.get(0)));
        assert_eq!(
            resolved(0x4F66C4).addr() as usize,
            JadeFont::home() + 0x1400
        );
        // Anything else is left as it was.
        assert_eq!(resolved(0x4F66DD + 0x20).addr(), 0x4F66DD + 0x20);
        assert_eq!(resolved(0x4F52C3).addr(), 0x4F52C3);
    }

    #[test]
    fn places_are_kept_apart_and_given_back_by_list() {
        let _lock = crate::store::test_lock();
        let mut memory = vec![0u8; size_of::<crate::memory::MemoryList>()];
        let _region = crate::pointers::Lent::new(&memory);
        let list =
            crate::pointers::GamePtr::new(crate::pointers::game_address(memory.as_mut_ptr()));
        let p = new_palette(list).unwrap();
        let t = new_table(list).unwrap();
        assert!(is_place(p) && is_place(t) && p < TABLE_FIRST && t >= TABLE_FIRST);
        write_bytes(p + 0x180, 4, &[1, 2], "a palette").unwrap();
        assert_eq!(bytes(p, "a palette").unwrap().byte(0x184), Some(1));
        assert_eq!(read_bytes(p + 0x180, 4, 2, "a palette").unwrap(), [1, 2]);
        assert!(write_bytes(p, 0x3FF, &[0, 0], "a palette").is_err());
        write_bytes(t, 0, &[9; 8], "a table").unwrap();
        write_bytes(t, 8, &[7; 4], "a table").unwrap();
        let held = bytes(t, "a table").unwrap();
        assert_eq!((held.byte(11), held.byte(12)), (Some(7), None));
        give_back(list);
        assert!(bytes(p, "a palette").is_err());
        assert!(bytes(t, "a table").is_err());
    }
}
