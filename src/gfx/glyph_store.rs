//! The fonts' and the sprites' palettes and glyph tables, Rust's (the font model).
//! The game carved them from heap blocks (`build_fonts`, `build_game_fonts`,
//! `build_glyphs`, `build_menu_sprites`, the installer's bar glyphs) and held their
//! addresses in the font objects (+0x14 palette, +0x18 glyph table), the sprite tables'
//! variables and the screens' palettes. Here each is bytes of its own, named by a place
//! below 0x10000, where no game address lies:
//! - a palette (0x400 bytes, 256 pixels): `id << 10 | offset`, ids 1 .. 15, so a place in
//!   it (the fonts' `palettes + 0x180`) is the palette's number plus the offset, as the
//!   address was;
//! - the blended font's palette (`BLENDED_PLACE`, id 16): the bytes of
//!   `text::blend_font::BLENDED_PALETTE`, which the installer's bar shares;
//! - the replay viewer's panel fonts' palette (`VIEWER_PALETTE`, id 17), the store's own,
//!   zeroed on first use and never given back;
//! - the loading font's palette (`SHEET_PLACE`, id 18, font 56): the bytes of
//!   `FontBlock.palette_sheet`, which `gfx::build_fonts` fills;
//! - a glyph table: `TABLE_FIRST + id`, growing as sheets are appended;
//! - the jade font (`JADE`, read only): its glyph table at `JADE`, its colour at
//!   `JADE_COLOUR`, the bytes of `JadeFont` on from each.
//!
//! Every glyph table and palette a font object holds is such a place (or 0, none, or a
//! counter in the measuring mode); no font object holds a game address.
//!
//! The lists keep their counts (`memory::owned::count_on` in place of `allocate`, the
//! cursor carved as before): running out is the original's fatal error at the same point.
//! What lies on a list is given back by its `free_list`.

use crate::errors::BadInput;
use crate::gfx::jade_store::JadeFont;
use crate::gfx::owner::FontStores;
use rc2k_core::strings::bytes::Bytes as _;
use std::sync::{Arc, Mutex};

/// A palette's bytes.
pub const PALETTE_BYTES: usize = 0x400;
const PALETTE_IDS: usize = 18;
/// The palettes from this id on belong to the store itself, not to a list.
const FIRST_OWN_PALETTE: usize = 17;
/// The palettes `new_palette` hands out are the ids below this one.
const FIRST_FIXED_PALETTE: usize = 16;
/// The blended font's palette (15 entries are made, `text::blend_font`).
pub const BLENDED_PLACE: u32 = 16 << 10;
/// The replay viewer's panel fonts' palette (`race::viewer_palettes`).
pub const VIEWER_PALETTE: u32 = 17 << 10;
/// The loading font's palette: the fonts block's palette sheet (`gfx::build_fonts`).
pub const SHEET_PLACE: u32 = 18 << 10;
const TABLE_FIRST: u32 = 19 << 10;
/// The jade font's glyph table; the glyph tables end below it.
pub const JADE: u32 = 0xA000;
/// The jade font's colour, 0x1400 bytes on in `JadeFont`.
pub const JADE_COLOUR: u32 = JADE + 0x1400;
/// `JadeFont`'s bytes from the glyph table to the end of the record.
const JADE_LEN: u32 = 0x1400 + 4 + 21 + 3;

struct Held {
    /// The list it is counted on (`give_back`'s key); none for the store's own palettes.
    list: Option<usize>,
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

/// The held bytes and the offset `place` names in them.
fn slot(s: &mut Store, place: u32) -> Option<(&mut Arc<Vec<u8>>, usize)> {
    if place >= TABLE_FIRST {
        let id = (place - TABLE_FIRST) as usize;
        let held = s.tables.get_mut(id)?.as_mut()?;
        Some((&mut held.bytes, 0))
    } else {
        let id = (place >> 10) as usize;
        let cell = s.palettes.get_mut(id)?;
        if cell.is_none() && id >= FIRST_OWN_PALETTE {
            *cell = Some(Held {
                list: None,
                bytes: Arc::new(vec![0; PALETTE_BYTES]),
            });
        }
        Some((&mut cell.as_mut()?.bytes, (place & 0x3FF) as usize))
    }
}

/// A new palette, zeroed, counted on nothing (the caller has carved its bytes off `list`):
/// its place. Refused when all fifteen are held.
pub fn new_palette(list: &dyn crate::cells::Store) -> Result<u32, BadInput> {
    let mut s = store();
    let id = (1..FIRST_FIXED_PALETTE)
        .find(|&id| s.palettes[id].is_none())
        .ok_or(BadInput("more than fifteen palettes held"))?;
    s.palettes[id] = Some(Held {
        list: Some(key(list)),
        bytes: Arc::new(vec![0; PALETTE_BYTES]),
    });
    Ok((id as u32) << 10)
}

/// A new glyph table, empty: its place. Refused past the places there are.
pub fn new_table(list: &dyn crate::cells::Store) -> Result<u32, BadInput> {
    let mut s = store();
    let held = Some(Held {
        list: Some(key(list)),
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
    if place >= JADE {
        s.tables[id] = None;
        return Err(BadInput("more glyph tables held than there are places"));
    }
    Ok(place)
}

/// What names a list: where its store is.
pub fn key(list: &dyn crate::cells::Store) -> usize {
    list.first().addr()
}

/// Everything held on the list `list` (its `key`) given back (`memory::free_list`).
pub fn give_back(list: usize) {
    let mut s = store();
    let Store { palettes, tables } = &mut *s;
    for slot in palettes.iter_mut().chain(tables.iter_mut()) {
        if slot.as_ref().is_some_and(|h| h.list == Some(list)) {
            *slot = None;
        }
    }
}

/// Where in the jade font's record `place` is, if it names a byte of it.
fn jade_offset(place: u32) -> Option<usize> {
    (JADE..JADE + JADE_LEN)
        .contains(&place)
        .then(|| (place - JADE) as usize)
}

/// A palette held in cells of another store.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellPalette {
    /// `text::blend_font::BLENDED_PALETTE` (`BLENDED_PLACE`).
    Blended,
    /// `FontBlock.palette_sheet` (`SHEET_PLACE`).
    Sheet,
}

impl CellPalette {
    fn cells(self, fonts: &FontStores) -> &[crate::session::PlainDword; 256] {
        match self {
            CellPalette::Blended => crate::text::blend_font::BLENDED_PALETTE.get(),
            CellPalette::Sheet => &fonts.font_block.palette_sheet,
        }
    }

    fn name(self) -> &'static str {
        match self {
            CellPalette::Blended => "a read past the blended palette",
            CellPalette::Sheet => "a read past the palette sheet",
        }
    }
}

/// Whether `place` is in a palette held in cells, which one and the offset in it.
fn cell_palette_offset(place: u32) -> Option<(CellPalette, usize)> {
    let which = match place & !0x3FF {
        BLENDED_PLACE => CellPalette::Blended,
        SHEET_PLACE => CellPalette::Sheet,
        _ => return None,
    };
    Some((which, (place & 0x3FF) as usize))
}

/// Byte `at` of a cell palette's 256 pixels, read from its cells.
fn cell_palette_byte(cells: &[crate::session::PlainDword; 256], at: usize) -> Option<u8> {
    let cell = cells.get(at / 4)?;
    Some(cell.get().to_le_bytes()[at % 4])
}

/// Byte `at` of the jade font's record (the glyph table, the colour, the file name and the
/// padding), read from its cells.
fn jade_byte(jade: &JadeFont, at: usize) -> Option<u8> {
    match at {
        0..0x1400 => Some(jade.glyphs.get(at)),
        0x1400..0x1404 => Some(jade.colour.get().to_le_bytes()[at - 0x1400]),
        0x1404..0x1419 => Some(jade.name.get(at - 0x1404)),
        0x1419..0x141C => Some(jade._tail.get(at - 0x1419)),
        _ => None,
    }
}

/// The bytes from a place on.
#[derive(Clone)]
pub enum Glyphs<'a> {
    Held(Arc<Vec<u8>>, usize),
    /// From this offset in the jade font's record, read from its cells.
    Jade(&'a JadeFont, usize),
    /// From this offset in a palette held in cells.
    Cells(CellPalette, &'a [crate::session::PlainDword; 256], usize),
}

impl rc2k_core::strings::bytes::Bytes for Glyphs<'_> {
    fn byte(&self, at: usize) -> Option<u8> {
        match self {
            Glyphs::Held(b, from) => b.get(from.checked_add(at)?).copied(),
            Glyphs::Jade(jade, from) => jade_byte(jade, from.checked_add(at)?),
            Glyphs::Cells(_, cells, from) => cell_palette_byte(cells, from.checked_add(at)?),
        }
    }
    fn copy_run(&self, at: usize, out: &mut [u8]) -> Option<()> {
        match self {
            Glyphs::Held(..) => {
                out.copy_from_slice(self.run(at, out.len())?);
                Some(())
            }
            _ => {
                for (i, place) in out.iter_mut().enumerate() {
                    *place = self.byte(at.checked_add(i)?)?;
                }
                Some(())
            }
        }
    }
    fn run(&self, at: usize, len: usize) -> Option<&[u8]> {
        let Glyphs::Held(b, from) = self else {
            return None;
        };
        let start = from.checked_add(at)?;
        b.get(start..start.checked_add(len)?)
    }
}

/// The bytes from `place` on: a palette or table of the store, or the jade font's.
pub fn bytes<'a>(fonts: &'a FontStores, place: u32, _what: &str) -> Result<Glyphs<'a>, BadInput> {
    if let Some(from) = jade_offset(place) {
        return Ok(Glyphs::Jade(&fonts.jade, from));
    }
    if let Some((which, from)) = cell_palette_offset(place) {
        return Ok(Glyphs::Cells(which, which.cells(fonts), from));
    }
    let mut s = store();
    let (b, from) = slot(&mut s, place).ok_or(BadInput("no palette or glyph table there"))?;
    Ok(Glyphs::Held(Arc::clone(b), from))
}

/// `len` bytes from `place` + `at`.
pub fn read_bytes(
    fonts: &FontStores,
    place: u32,
    at: u32,
    len: usize,
    _what: &str,
) -> Result<Vec<u8>, BadInput> {
    let view = if let Some(from) = jade_offset(place) {
        Some((Glyphs::Jade(&fonts.jade, from), "a read past the jade font"))
    } else {
        cell_palette_offset(place)
            .map(|(which, from)| (Glyphs::Cells(which, which.cells(fonts), from), which.name()))
    };
    if let Some((view, refusal)) = view {
        let mut out = vec![0; len];
        view.copy_run(at as usize, &mut out)
            .ok_or(BadInput(refusal))?;
        return Ok(out);
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
pub fn write_bytes(
    fonts: &FontStores,
    place: u32,
    at: u32,
    bytes: &[u8],
    _what: &str,
) -> Result<(), BadInput> {
    if jade_offset(place).is_some() {
        return Err(BadInput("a write into the jade font"));
    }
    if let Some((which, from)) = cell_palette_offset(place) {
        let cells = which.cells(fonts);
        let start = from + at as usize;
        if start + bytes.len() > PALETTE_BYTES {
            return Err(BadInput("a write past a palette"));
        }
        // Whole dwords only: the palette is written entry by entry.
        if !start.is_multiple_of(4) || !bytes.len().is_multiple_of(4) {
            return Err(BadInput("a write into a cell palette off its entries"));
        }
        for (n, entry) in bytes.as_chunks::<4>().0.iter().enumerate() {
            cells[start / 4 + n].set(u32::from_le_bytes(*entry));
        }
        return Ok(());
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

    #[test]
    fn the_jade_font_is_read_through_its_places() {
        let fonts = &crate::gfx::owner::gfx().fonts;
        let _lock = crate::test_support::test_lock();
        // The glyph table and the palette (the colour), as the menu fonts name them.
        let table = bytes(fonts, JADE, "the jade glyphs").unwrap();
        let colour = read_bytes(fonts, JADE_COLOUR, 0, 4, "the jade colour").unwrap();
        assert_eq!(colour, fonts.jade.colour.get().to_le_bytes());
        assert_eq!(table.byte(0), Some(fonts.jade.glyphs.get(0)));
        assert_eq!(table.byte(0x1400), Some(colour[0]));
        assert!(write_bytes(fonts, JADE_COLOUR, 0, &[1], "the jade colour").is_err());
        assert!(bytes(fonts, JADE + JADE_LEN, "past the jade font").is_err());
    }

    #[test]
    fn the_stores_own_palettes_start_zeroed() {
        let fonts = &crate::gfx::owner::gfx().fonts;
        let _lock = crate::test_support::test_lock();
        for place in [BLENDED_PLACE, VIEWER_PALETTE] {
            assert_eq!(
                read_bytes(fonts, place, 0x3FC, 4, "a palette").unwrap(),
                [0; 4]
            );
            write_bytes(fonts, place, 8, &[3, 0, 0, 0], "a palette").unwrap();
            assert_eq!(
                read_bytes(fonts, place, 8, 4, "a palette").unwrap(),
                [3, 0, 0, 0]
            );
            write_bytes(fonts, place, 8, &[0; 4], "a palette").unwrap();
        }
    }

    #[test]
    fn places_are_kept_apart_and_given_back_by_list() {
        let fonts = &crate::gfx::owner::gfx().fonts;
        let _lock = crate::test_support::test_lock();
        let list = crate::memory::ListCells::new(crate::memory::MemoryList {
            head: 0,
            total: 0,
            cursor: 0,
            remaining: 0,
        });
        let key = key(&list);
        let p = new_palette(&list).unwrap();
        let t = new_table(&list).unwrap();
        assert!(p < TABLE_FIRST && t >= TABLE_FIRST);
        write_bytes(fonts, p + 0x180, 4, &[1, 2], "a palette").unwrap();
        assert_eq!(bytes(fonts, p, "a palette").unwrap().byte(0x184), Some(1));
        assert_eq!(
            read_bytes(fonts, p + 0x180, 4, 2, "a palette").unwrap(),
            [1, 2]
        );
        assert!(write_bytes(fonts, p, 0x3FF, &[0, 0], "a palette").is_err());
        write_bytes(fonts, t, 0, &[9; 8], "a table").unwrap();
        write_bytes(fonts, t, 8, &[7; 4], "a table").unwrap();
        let held = bytes(fonts, t, "a table").unwrap();
        assert_eq!((held.byte(11), held.byte(12)), (Some(7), None));
        give_back(key);
        assert!(bytes(fonts, p, "a palette").is_err());
        assert!(bytes(fonts, t, "a table").is_err());
    }
}
