//! The menu items' sprite lists, out of RAL.EXE's image (IDA 0x747998 .. 0x747AD8; phase 7,
//! typed): 16 lists of five sprite numbers (the item's look in each state), which a menu
//! item names by address (`menus_store::MenuItem::sprites`; `moved::point_into_homes`
//! makes those addresses real). The first eight are the menus' (sprites 0xA ..), the last
//! eight the in-race menus' (0x11 ..). Nothing changes them.

use crate::session::Dword;

#[repr(C)]
pub struct ItemSprites {
    pub lists: [[Dword; 5]; 16],
}

pub static ITEM_SPRITES: ItemSprites = ItemSprites {
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
};

/// IDA 0x747998, the first byte of the piece.
pub const START: usize = 0x747998;
/// IDA 0x747AD8, the end of the piece.
pub const END: usize = 0x747AD8;

/// Where IDA 0x747998 is now (`moved::MOVED`'s home).
pub fn item_sprites_addr() -> u32 {
    crate::pointers::game_address(&ITEM_SPRITES as *const ItemSprites)
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
                crate::pointers::host(item_sprites_addr()) as *const u8,
                END - START,
            )
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
    }
}
