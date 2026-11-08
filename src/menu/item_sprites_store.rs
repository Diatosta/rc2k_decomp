//! The menu items' sprite lists, out of RAL.EXE's image (IDA 0x747998 .. 0x747AD8; phase 7,
//! typed): 16 lists of five sprite numbers (the item's look in each state), which a menu
//! item names (`menus_store::MenuItem::sprites`, a `SpriteCell`: the original kept the
//! list's address, `moved::point_into_homes` passes those places over). The first eight are the menus' (sprites 0xA ..), the last
//! eight the in-race menus' (0x11 ..). Nothing changes them.

use crate::menu::owner::menus;
use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ItemSprites {
    pub lists: [[Dword; 5]; 16],
}

impl ItemSprites {
    pub const fn new() -> Self {
        ItemSprites {
            lists: [
                [
                    Dword::new(0xa),
                    Dword::new(0xc),
                    Dword::new(0xa),
                    Dword::new(0xc),
                    Dword::new(0xb),
                ],
                [
                    Dword::new(0xa),
                    Dword::new(0xc),
                    Dword::new(0xf),
                    Dword::new(0x10),
                    Dword::new(0xb),
                ],
                [
                    Dword::new(0xa),
                    Dword::new(0xc),
                    Dword::new(0xf),
                    Dword::new(0xe),
                    Dword::new(0xb),
                ],
                [
                    Dword::new(0xa),
                    Dword::new(0xc),
                    Dword::new(0xd),
                    Dword::new(0x10),
                    Dword::new(0xb),
                ],
                [
                    Dword::new(0xa),
                    Dword::new(0xc),
                    Dword::new(0xa),
                    Dword::new(0xe),
                    Dword::new(0xb),
                ],
                [
                    Dword::new(0xa),
                    Dword::new(0xc),
                    Dword::new(0xd),
                    Dword::new(0xc),
                    Dword::new(0xb),
                ],
                [
                    Dword::new(0xa),
                    Dword::new(0xc),
                    Dword::new(0xf),
                    Dword::new(0xc),
                    Dword::new(0xb),
                ],
                [
                    Dword::new(0xa),
                    Dword::new(0xc),
                    Dword::new(0xa),
                    Dword::new(0x10),
                    Dword::new(0xb),
                ],
                [
                    Dword::new(0x11),
                    Dword::new(0x13),
                    Dword::new(0x11),
                    Dword::new(0x13),
                    Dword::new(0x12),
                ],
                [
                    Dword::new(0x11),
                    Dword::new(0x13),
                    Dword::new(0x16),
                    Dword::new(0x17),
                    Dword::new(0x12),
                ],
                [
                    Dword::new(0x11),
                    Dword::new(0x13),
                    Dword::new(0x16),
                    Dword::new(0x15),
                    Dword::new(0x12),
                ],
                [
                    Dword::new(0x11),
                    Dword::new(0x13),
                    Dword::new(0x14),
                    Dword::new(0x17),
                    Dword::new(0x12),
                ],
                [
                    Dword::new(0x11),
                    Dword::new(0x13),
                    Dword::new(0x11),
                    Dword::new(0x15),
                    Dword::new(0x12),
                ],
                [
                    Dword::new(0x11),
                    Dword::new(0x13),
                    Dword::new(0x14),
                    Dword::new(0x13),
                    Dword::new(0x12),
                ],
                [
                    Dword::new(0x11),
                    Dword::new(0x13),
                    Dword::new(0x16),
                    Dword::new(0x13),
                    Dword::new(0x12),
                ],
                [
                    Dword::new(0x11),
                    Dword::new(0x13),
                    Dword::new(0x11),
                    Dword::new(0x17),
                    Dword::new(0x12),
                ],
            ],
        }
    }
}

/// What a menu item's +0x20 holds: a sprite number (the glyph drawn behind an item that is no
/// button, and its palette), or for a button one of the lists (the original kept its
/// address).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ItemSprite {
    Sprite(u32),
    List(usize),
}

impl ItemSprite {
    /// What the original held: the number, or the list's address.
    pub fn raw(self) -> u32 {
        match self {
            Self::Sprite(n) => n,
            Self::List(n) => {
                crate::pointers::GamePtr::to(&menus().panels.item_sprites.lists[n]).addr()
            }
        }
    }
}

/// A list kept in a `SpriteCell`: `LIST_TAG | n` (the numbers the game keeps there are
/// 0 .. 2 and 0xFFFFFFA9; one in the tag's range is refused when set).
const LIST_TAG: u32 = 0x7FFF_FFF0;

/// A menu item's +0x20 (phase 13, handles).
#[repr(transparent)]
#[derive(rc2k_derive::Cells)]
pub struct SpriteCell(crate::session::Dword);

impl SpriteCell {
    /// From the image's value (`ida`: a number, or IDA's address of a list; an address inside
    /// the lists that is no list's start, or a number in the tag's range, fails to compile).
    pub const fn at(ida: u32) -> Self {
        let ida_at = ida as usize;
        let value = if START <= ida_at && ida_at < END {
            assert!((ida_at - START).is_multiple_of(0x14));
            LIST_TAG | ((ida_at - START) / 0x14) as u32
        } else {
            assert!(ida & !0xF != LIST_TAG);
            ida
        };
        Self(crate::session::Dword::new(value))
    }

    pub fn get(&self) -> ItemSprite {
        let v = self.0.get();
        if v & !0xF == LIST_TAG {
            ItemSprite::List((v & 0xF) as usize)
        } else {
            ItemSprite::Sprite(v)
        }
    }

    pub fn set(&self, sprite: ItemSprite) -> Result<(), crate::errors::BadInput> {
        let v = match sprite {
            ItemSprite::Sprite(n) if n & !0xF == LIST_TAG => {
                return Err(crate::errors::BadInput(
                    "a sprite number in the lists' tag range",
                ));
            }
            ItemSprite::Sprite(n) => n,
            ItemSprite::List(n) if n < 16 => LIST_TAG | n as u32,
            ItemSprite::List(_) => return Err(crate::errors::BadInput("no such sprite list")),
        };
        self.0.set(v);
        Ok(())
    }
}

/// IDA 0x747998, the first byte of the piece.
pub const START: usize = 0x747998;
/// IDA 0x747AD8, the end of the piece.
pub const END: usize = 0x747AD8;

/// Where IDA 0x747998 is now (`moved::MOVED`'s home).
#[cfg(test)]
pub fn item_sprites_addr() -> u32 {
    crate::pointers::game_address(&raw const crate::menu::owner::MENUS.panels.item_sprites)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::size_of;

    #[test]
    fn item_sprites_start_as_the_games() {
        assert_eq!(size_of::<ItemSprites>(), END - START);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host(item_sprites_addr()).cast_const(),
                END - START,
            )
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
    }

    #[test]
    fn a_sprite_cell_keeps_numbers_and_lists() {
        let c = SpriteCell::at(0x7479AC);
        assert_eq!(c.get(), ItemSprite::List(1));
        assert_eq!(SpriteCell::at(2).get(), ItemSprite::Sprite(2));
        assert_eq!(
            SpriteCell::at(0xFFFF_FFA9).get(),
            ItemSprite::Sprite(0xFFFF_FFA9)
        );
        c.set(ItemSprite::Sprite(0xFFFF_FFA9)).unwrap();
        assert_eq!(c.get(), ItemSprite::Sprite(0xFFFF_FFA9));
        assert!(c.set(ItemSprite::Sprite(LIST_TAG | 3)).is_err());
        assert!(c.set(ItemSprite::List(16)).is_err());
        assert_eq!(c.get(), ItemSprite::Sprite(0xFFFF_FFA9));
    }
}
