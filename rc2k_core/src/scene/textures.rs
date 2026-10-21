//! The scene's textures: which are needed at which level, which slot shows which, and
//! the texture a polygon's word names (converted 2026-10-21 from `scene::no_mips_for`
//! 0x46512F, `forget_slot_texture` 0x44EB66, `raise_texture_needs` 0x423F97,
//! `set_texture_levels` 0x422426, `count_texture_needs` 0x423EB1, `map_texture_places`
//! 0x42570D, `look_up_texture` 0x44E668, `assign_kind_textures` 0x4256D8,
//! `assign_menu_textures` 0x490DF3 and `texture_picture_folder` 0x425A59).
//!
//! A level is an index into the table of what a texture of each level takes in k; -1 is
//! "not needed". The tables are 0x400 entries, one a texture (or a slot); a texture's
//! place is one of 0xFA0.

use crate::Invalid;

/// Textures, and texture slots.
pub const SLOTS: u32 = 0x400;
/// Places a texture can have.
pub const PLACES: u32 = 0xFA0;
/// The levels the sizes' table has.
pub const LEVELS: u32 = 13;
/// "Not needed", "not loaded", "no place".
pub const NONE: u32 = u32::MAX;
/// `g_polygonTexture` for a word that names no loaded texture.
pub const NO_TEXTURE: u32 = 0xFFFF_FC43;

/// `no_mips_for`: whether a texture of this size gets no mip levels (0x80 .. 0xFF, and
/// 0x140 on, unsigned).
pub fn no_mips_for(size: u32) -> bool {
    (0x80..0x100).contains(&size) || size >= 0x140
}

/// `forget_slot_texture`: the slot's loaded texture forgotten, so that it is loaded again.
/// Refuses a slot past the table.
pub fn forget_slot_texture(loaded: &mut [u32], slot: u32) -> Result<(), Invalid> {
    let entry = loaded
        .get_mut(slot as usize)
        .ok_or(Invalid("a texture slot past 0x3FF"))?;
    *entry = NONE;
    Ok(())
}

/// The three tables of what the textures need: each one's tag (which set of kinds it
/// belongs to) and its level in system and in video memory.
pub struct Needs<'a> {
    pub tags: &'a [u32],
    pub system: &'a mut [u32],
    pub video: &'a mut [u32],
}

impl Needs<'_> {
    /// `raise_texture_needs`: every texture of the tag needed at `level` at least (above
    /// its level, unsigned, so "not needed" is never raised); whether any entry changed.
    pub fn raise(&mut self, level: u32, tag: u32) -> bool {
        let mut changed = false;
        for (n, &t) in self.tags.iter().enumerate() {
            if t != tag {
                continue;
            }
            for table in [&mut *self.system, &mut *self.video] {
                if level > table[n] {
                    table[n] = level;
                    changed = true;
                }
            }
        }
        changed
    }
}

/// `set_texture_levels`' tables: the levels in both memories and the tags, all written.
pub struct LevelTables<'a> {
    pub tags: &'a mut [u32],
    pub system: &'a mut [u32],
    pub video: &'a mut [u32],
}

impl LevelTables<'_> {
    /// `set_texture_levels`: `count` slots from `first` get `level` in both tables and the
    /// tag. Refuses no slots (the original counts round) or slots past the tables, before
    /// writing.
    pub fn set_run(&mut self, first: u32, count: u32, level: u32, tag: u32) -> Result<(), Invalid> {
        if count == 0 || first >= SLOTS || count > SLOTS - first {
            return Err(Invalid("no texture slots, or past 0x400"));
        }
        let run = first as usize..(first + count) as usize;
        self.system[run.clone()].fill(level);
        self.video[run.clone()].fill(level);
        self.tags[run].fill(tag);
        Ok(())
    }
}

/// `count_texture_needs`' totals: k and textures in each memory.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NeedTotals {
    pub video_k: u32,
    pub system_k: u32,
    pub video_textures: u32,
    pub system_textures: u32,
}

impl NeedTotals {
    /// In the order of the game's four variables (from `dword_5DC968`).
    pub fn to_array(self) -> [u32; 4] {
        [
            self.video_k,
            self.system_k,
            self.video_textures,
            self.system_textures,
        ]
    }
}

/// `count_texture_needs`: the needed textures counted and their levels' sizes (`size_of`,
/// asked per needed level) added, system and video memory taken texture by texture; and
/// the last number looked at (a level or a size: eax). Refuses a level past the sizes'
/// table.
pub fn count_needs(
    system: &[u32],
    video: &[u32],
    size_of: impl Fn(usize) -> u32,
) -> Result<(NeedTotals, u32), Invalid> {
    let mut t = NeedTotals::default();
    let mut last = 0;
    for (&s, &v) in system.iter().zip(video) {
        for (level, system) in [(s, true), (v, false)] {
            last = level;
            if level == NONE {
                continue;
            }
            if level >= LEVELS {
                return Err(Invalid("a texture's level past the sizes' table"));
            }
            last = size_of(level as usize);
            if system {
                t.system_textures += 1;
                t.system_k = t.system_k.wrapping_add(last);
            } else {
                t.video_textures += 1;
                t.video_k = t.video_k.wrapping_add(last);
            }
        }
    }
    Ok((t, last))
}

/// `map_texture_places`: each place's slot (`places`, all -1 first): for the slots from
/// the last down, the texture it wants, if a texture, has a place, and that place, if
/// one, gets the slot (so of two slots the lower stays). Gives the last number looked at
/// (eax).
pub fn map_places(wanted: &[u32], texture_place: &[u32], places: &mut [u32]) -> u32 {
    places.fill(NONE);
    let mut last = NONE;
    for (slot, &w) in wanted.iter().enumerate().rev() {
        last = w;
        if w >= SLOTS {
            continue;
        }
        last = texture_place[w as usize];
        if let Some(p) = places.get_mut(last as usize) {
            *p = slot as u32;
        }
    }
    last
}

/// What `look_up_texture` gives: `g_polygonTexture` and eax.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LookUp {
    pub texture: u32,
    pub eax: u32,
}

/// `look_up_texture`: the slot a texture word names: -1 none; with the top bit a place
/// (`place_slot`, below 0xFA0); else a kind's first slot (`first_slot`, below 0x400); a
/// slot not below the slots loaded (`slot_count`, read only then) none.
pub fn look_up(
    word: u32,
    place_slot: impl FnOnce(usize) -> u32,
    first_slot: impl FnOnce(usize) -> u32,
    slot_count: impl FnOnce() -> u32,
) -> LookUp {
    let none = |eax| LookUp {
        texture: NO_TEXTURE,
        eax,
    };
    if word == NONE {
        return none(word);
    }
    let slot = if (word as i32) < 0 {
        let place = word & 0x7FFF_FFFF;
        if place >= PLACES {
            return none(place);
        }
        place_slot(place as usize)
    } else {
        if word >= SLOTS {
            return none(word);
        }
        first_slot(word as usize)
    };
    let count = slot_count();
    if slot >= count || count == 0 {
        return none(slot);
    }
    LookUp {
        texture: slot,
        eax: slot,
    }
}

/// `assign_kind_textures`: each slot wants the texture of the kind it is the first slot
/// of (`wanted` all -1 first; the kinds from the last down, so of two kinds with one slot
/// the lower one's stays). Refuses a first slot past the table, before writing.
pub fn assign_kind_textures(
    first_slots: &[u32],
    first_textures: &[u32],
    wanted: &mut [u32],
) -> Result<(), Invalid> {
    if first_slots
        .iter()
        .any(|&s| s != NONE && s as usize >= wanted.len())
    {
        return Err(Invalid("a kind's first slot past the slots' table"));
    }
    wanted.fill(NONE);
    for (kind, &slot) in first_slots.iter().enumerate().rev() {
        if slot != NONE {
            wanted[slot as usize] = first_textures[kind];
        }
    }
    Ok(())
}

/// `assign_menu_textures`' wants (slot, texture): the menu kind's eight slots from its
/// first slot, each of its own numbers that is not -1 plus the kind's first texture.
/// Refuses a slot past the table.
pub fn menu_textures(
    own: [u32; 8],
    first_slot: u32,
    first_texture: u32,
) -> Result<Vec<(u32, u32)>, Invalid> {
    let wanted: Vec<(u32, u32)> = (0..8u32)
        .filter(|&n| own[n as usize] != NONE)
        .map(|n| {
            (
                first_slot.wrapping_add(n),
                own[n as usize].wrapping_add(first_texture),
            )
        })
        .collect();
    if wanted.iter().any(|&(slot, _)| slot >= SLOTS) {
        return Err(Invalid("the menu's texture slots lie past the table"));
    }
    Ok(wanted)
}

/// The folders textures' pictures are read from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PictureFolder {
    /// `files\a\`: below 900, and 902 .. 999.
    A,
    /// `files\m\`: 900 and 901.
    M,
    /// `files\b\`: 1000 .. 1999.
    B,
    /// `files\g\frnt\`: 2000 .. 2999.
    Front,
    /// `files\s\extr\`: 3000 on.
    Extra,
}

/// `texture_picture_folder`: a texture's picture's folder by its number.
pub fn picture_folder(number: u32) -> PictureFolder {
    match number {
        0..900 | 902..1000 => PictureFolder::A,
        900..902 => PictureFolder::M,
        1000..2000 => PictureFolder::B,
        2000..3000 => PictureFolder::Front,
        _ => PictureFolder::Extra,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small() {
        assert_eq!(
            [0, 0x7F, 0x80, 0xFF, 0x100, 0x13F, 0x140, u32::MAX].map(no_mips_for),
            [false, false, true, true, false, false, true, true]
        );
        let mut loaded = [1u32; 4];
        assert!(forget_slot_texture(&mut loaded, 4).is_err());
        assert_eq!(forget_slot_texture(&mut loaded, 2), Ok(()));
        assert_eq!(loaded, [1, 1, NONE, 1]);
    }

    #[test]
    fn needs() {
        let tags = [1, 2, 1, 1];
        let (mut system, mut video) = ([0, 0, NONE, 5], [3, 0, 1, 1]);
        let mut n = Needs {
            tags: &tags,
            system: &mut system,
            video: &mut video,
        };
        assert!(n.raise(2, 1));
        assert!(!n.raise(2, 1));
        assert_eq!((system, video), ([2, 0, NONE, 5], [3, 0, 2, 2]));
        let sizes = |l: usize| 1 << l;
        let (t, last) = count_needs(&system, &video, sizes).unwrap();
        assert_eq!(t.to_array(), [8 + 1 + 4 + 4, 4 + 32 + 1, 4, 3]);
        assert_eq!(last, 4);
        assert!(count_needs(&[LEVELS], &[0], sizes).is_err());
        assert_eq!(count_needs(&[0], &[NONE], sizes).unwrap().1, NONE);
    }

    #[test]
    fn runs() {
        let (mut tags, mut system, mut video) = ([0u32; 0x400], [0u32; 0x400], [0u32; 0x400]);
        let mut t = LevelTables {
            tags: &mut tags,
            system: &mut system,
            video: &mut video,
        };
        assert!(t.set_run(5, 0, 1, 1).is_err());
        assert!(t.set_run(0x3FF, 2, 1, 1).is_err());
        assert_eq!(t.set_run(0x3FE, 2, 3, 0x63), Ok(()));
        assert_eq!(
            (tags[0x3FD], tags[0x3FE], system[0x3FF], video[0x3FE]),
            (0, 0x63, 3, 3)
        );
    }

    #[test]
    fn places() {
        let mut wanted = vec![NONE; SLOTS as usize];
        let mut place_of = vec![NONE; SLOTS as usize];
        wanted[3] = 7;
        wanted[9] = 7;
        wanted[0] = 8;
        place_of[7] = 0x20;
        place_of[8] = PLACES;
        let mut places = vec![0u32; PLACES as usize];
        assert_eq!(map_places(&wanted, &place_of, &mut places), PLACES);
        assert_eq!(places[0x20], 3);
        assert_eq!(places.iter().filter(|&&p| p != NONE).count(), 1);
    }

    #[test]
    fn assigns() {
        let mut wanted = [7u32; 4];
        assert!(assign_kind_textures(&[NONE, 4], &[1, 2], &mut wanted).is_err());
        assert_eq!(wanted, [7; 4]);
        assign_kind_textures(&[2, NONE, 2, 0], &[10, 11, 12, 13], &mut wanted).unwrap();
        assert_eq!(wanted, [13, NONE, 10, NONE]);
        let own = [0, NONE, 2, NONE, NONE, NONE, NONE, 1];
        assert_eq!(
            menu_textures(own, 0x3F0, 5),
            Ok(vec![(0x3F0, 5), (0x3F2, 7), (0x3F7, 6)])
        );
        assert!(menu_textures(own, 0x3F9, 5).is_err());
        assert!(menu_textures([NONE; 8], u32::MAX, 5).unwrap().is_empty());
        assert_eq!(
            [
                0,
                899,
                900,
                901,
                902,
                999,
                1000,
                1999,
                2000,
                2999,
                3000,
                u32::MAX
            ]
            .map(picture_folder),
            [
                PictureFolder::A,
                PictureFolder::A,
                PictureFolder::M,
                PictureFolder::M,
                PictureFolder::A,
                PictureFolder::A,
                PictureFolder::B,
                PictureFolder::B,
                PictureFolder::Front,
                PictureFolder::Front,
                PictureFolder::Extra,
                PictureFolder::Extra
            ]
        );
    }

    #[test]
    fn look_ups() {
        let l = |w| look_up(w, |p| p as u32 + 1, |k| k as u32 * 2, || 0x10);
        assert_eq!(
            l(NONE),
            LookUp {
                texture: NO_TEXTURE,
                eax: NONE
            }
        );
        assert_eq!(l(0x8000_0003), LookUp { texture: 4, eax: 4 });
        assert_eq!(l(0x8000_0FA0).eax, 0xFA0);
        assert_eq!(l(0x400).texture, NO_TEXTURE);
        assert_eq!(
            l(7),
            LookUp {
                texture: 14,
                eax: 14
            }
        );
        assert_eq!(
            l(8),
            LookUp {
                texture: NO_TEXTURE,
                eax: 16
            }
        );
        assert_eq!(look_up(1, |_| 0, |_| 0, || 0).texture, NO_TEXTURE);
    }
}
