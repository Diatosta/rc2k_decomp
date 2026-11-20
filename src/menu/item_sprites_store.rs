//! The menu items' sprite lists: 16 lists of five sprite numbers (the item's look in each
//! state), which a menu item names (`menus_store::MenuItem::sprites`, a `SpriteCell`). The
//! first eight are the menus' (sprites 0xA ..), the last eight the in-race menus' (0x11 ..).
//! Read-only data.

/// The sprite lists: 16 lists of five sprite numbers each, by list number (`ItemSprite::List`).
pub static ITEM_SPRITES: [[u32; 5]; 16] = [
    [0xa, 0xc, 0xa, 0xc, 0xb],
    [0xa, 0xc, 0xf, 0x10, 0xb],
    [0xa, 0xc, 0xf, 0xe, 0xb],
    [0xa, 0xc, 0xd, 0x10, 0xb],
    [0xa, 0xc, 0xa, 0xe, 0xb],
    [0xa, 0xc, 0xd, 0xc, 0xb],
    [0xa, 0xc, 0xf, 0xc, 0xb],
    [0xa, 0xc, 0xa, 0x10, 0xb],
    [0x11, 0x13, 0x11, 0x13, 0x12],
    [0x11, 0x13, 0x16, 0x17, 0x12],
    [0x11, 0x13, 0x16, 0x15, 0x12],
    [0x11, 0x13, 0x14, 0x17, 0x12],
    [0x11, 0x13, 0x11, 0x15, 0x12],
    [0x11, 0x13, 0x14, 0x13, 0x12],
    [0x11, 0x13, 0x16, 0x13, 0x12],
    [0x11, 0x13, 0x11, 0x17, 0x12],
];

pub use rc2k_core::menu::tables::ItemSprite;

/// A list kept in a `SpriteCell`: `LIST_TAG | n` (the plain numbers kept there are small or
/// a few below zero; one in the tag's range is refused when set).
const LIST_TAG: u32 = 0x7FFF_FFF0;

/// What is drawn behind a menu item, as it changes in play.
#[repr(transparent)]
#[derive(rc2k_derive::Cells)]
pub struct SpriteCell(crate::memory::session::PlainDword);

impl SpriteCell {
    /// From the table's value (a list past the sixteen, or a number in the tag's range, fails
    /// to compile).
    pub const fn new(sprite: ItemSprite) -> Self {
        let value = match sprite {
            ItemSprite::List(n) => {
                assert!(n < ITEM_SPRITES.len());
                LIST_TAG | n as u32
            }
            ItemSprite::Sprite(n) => {
                assert!(n & !0xF != LIST_TAG);
                n
            }
        };
        Self(crate::memory::session::PlainDword::new(value))
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_sprite_cell_keeps_numbers_and_lists() {
        let c = SpriteCell::new(ItemSprite::List(1));
        assert_eq!(c.get(), ItemSprite::List(1));
        assert_eq!(
            SpriteCell::new(ItemSprite::Sprite(2)).get(),
            ItemSprite::Sprite(2)
        );
        assert_eq!(
            SpriteCell::new(ItemSprite::Sprite(0xFFFF_FFA9)).get(),
            ItemSprite::Sprite(0xFFFF_FFA9)
        );
        c.set(ItemSprite::Sprite(0xFFFF_FFA9)).unwrap();
        assert_eq!(c.get(), ItemSprite::Sprite(0xFFFF_FFA9));
        assert!(c.set(ItemSprite::Sprite(LIST_TAG | 3)).is_err());
        assert!(c.set(ItemSprite::List(16)).is_err());
        assert_eq!(c.get(), ItemSprite::Sprite(0xFFFF_FFA9));
    }
}
