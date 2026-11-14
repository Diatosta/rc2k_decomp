//! The carousels' set-ups that are only stores, one list a menu, as
//! `scene::set_up_carousel`'s per-menu functions make them. Every menu stores the same things in
//! the same order (`menu_ops`), only the numbers differ, so each is one `menu!` entry.
use crate::scene::set_up_carousel::Field;
use crate::scene::set_up_carousel::Op::{self, Fill, Run, Set};

/// The stores of one menu, in the order the game makes them: `set_up`, the fill, the
/// textures (`textures[k]` to `carousel_textures[k]`), the front-end object's size, the
/// kinds (`kinds[k]` to `carousel.kinds[k]`), the count, the front, angle and direction (0
/// in all 17), the spacing, y, z and the forward step. `N` is the number of stores.
const fn menu_ops<const N: usize>(
    set_up: u32,
    textures: &[u32],
    size: u32,
    kinds: &[u32],
    count: u32,
    [spacing, y, z, forward]: [u32; 4],
) -> [Op; N] {
    let mut ops = [Fill; N];
    ops[0] = Set(Field::SetUp, set_up);
    let mut n = 2; // ops[1] is the fill
    let mut k = 0;
    while k < textures.len() {
        ops[n] = Set(Field::Texture(k), textures[k]);
        n += 1;
        k += 1;
    }
    ops[n] = Run(|| &crate::race::view_scratch::front_end().object.size, size);
    n += 1;
    k = 0;
    while k < kinds.len() {
        ops[n] = Set(Field::Kind(k), kinds[k]);
        n += 1;
        k += 1;
    }
    ops[n] = Set(Field::Count, count);
    ops[n + 1] = Set(Field::Front, 0);
    ops[n + 2] = Set(Field::Angle, 0);
    ops[n + 3] = Set(Field::Direction, 0);
    ops[n + 4] = Set(Field::Spacing, spacing);
    ops[n + 5] = Set(Field::Y, y);
    ops[n + 6] = Set(Field::Z, z);
    ops[n + 7] = Set(Field::Forward, forward);
    assert!(n + 8 == N);
    ops
}

/// One menu's list: `menu!(NAME, set_up, textures [..], size, kinds [..], count, [spacing,
/// y, z, forward])`.
macro_rules! menu {
    ($(#[$doc:meta])* $name:ident, $set_up:expr, textures [$($t:expr),+], $size:expr, kinds [$($k:expr),+], $count:expr, $tail:expr) => {
        $(#[$doc])*
        const $name: &[Op] = &menu_ops::<{ 11 + [$($t),+].len() + [$($k),+].len() }>(
            $set_up,
            &[$($t),+],
            $size,
            &[$($k),+],
            $count,
            $tail,
        );
    };
}

menu!(
    MENU_00, 1,
    textures [0xB, 0xC, 0x5, 0x7, 0x8, 0x9], 16,
    kinds [0x267, 0x269, 0x275, 0x262, 0x26D], 5,
    [0x214C0, 0xFFFF4000, 0x36F00, 0x20000]
);
menu!(
    MENU_01, 1,
    textures [0x6, 0x4, 0x13, 0x11, 0xE], 14,
    kinds [0x261, 0x25F, 0x278, 0x276, 0x26B, 0x26D], 6,
    [0x1D4C0, 0xFFFFD000, 0x34F00, 0x1D000]
);
menu!(
    MENU_02, 0,
    textures [0x5, 0x10], 16,
    kinds [0x270, 0x264, 0x26F, 0x265, 0x26C, 0x26D], 6,
    [0x214C0, 0xFFFF4000, 0x36F00, 0x20000]
);
menu!(
    MENU_03, 1,
    textures [0x14, 0xE, 0x7], 16,
    kinds [0x27B, 0x269, 0x26D], 3,
    [0x214C0, 0xFFFF4000, 0x36F00, 0x20000]
);
menu!(
    MENU_04, 1,
    textures [0x10, 0x5], 16,
    kinds [0x270, 0x264, 0x26D], 3,
    [0x214C0, 0xFFFF4000, 0x36F00, 0x20000]
);
menu!(
    MENU_05, 0,
    textures [0x0, 0x2, 0x3, 0x5], 16,
    kinds [0x259, 0x25A, 0x25B, 0x25C, 0x25D, 0x25E, 0x26D], 7,
    [0x214C0, 0xFFFF4000, 0x36F00, 0x20000]
);
menu!(
    MENU_06, 1,
    textures [0x15, 0x16, 0x8, 0x9, 0x5], 16,
    kinds [0x27D, 0x27C, 0x263, 0x26D], 4,
    [0x214C0, 0xFFFF4000, 0x36F00, 0x20000]
);
menu!(
    MENU_07, 1,
    textures [0x10, 0x11, 0xD, 0xA, 0x8, 0x17, 0x5, 0xF], 16,
    kinds [0x27A, 0x268, 0x26A, 0x266, 0x263, 0x279, 0x26D], 7,
    [0x235C0, 0xFFFF4000, 0x27900, 0x20000]
);
menu!(
    MENU_08, 1,
    textures [0x10, 0x5], 16,
    kinds [0x270, 0x271, 0x272, 0x273, 0x26D], 5,
    [0x235C0, 0xFFFF4000, 0x27900, 0x20000]
);
menu!(
    MENU_09, 1,
    textures [0x18, 0x17, 0xE, 0x12, 0x5], 16,
    kinds [0x274, 0x27E, 0x26B, 0x277, 0x26D], 5,
    [0x235C0, 0xFFFF4000, 0x27900, 0x20000]
);
menu!(
    MENU_0A, 1,
    textures [0xF, 0xC, 0xD, 0x7, 0x7, 0x13, 0x12], 16,
    kinds [0x26E, 0x269, 0x26A, 0x27B, 0x262, 0x279, 0x277, 0x27F], 8,
    [0x235C0, 0xFFFF4000, 0x27900, 0x20000]
);
menu!(
    MENU_0F, 1,
    textures [0xF, 0xD, 0x7, 0x5], 16,
    kinds [0x26E, 0x26A, 0x27B, 0x262, 0x26D], 5,
    [0x235C0, 0xFFFF4000, 0x27900, 0x20000]
);
menu!(
    MENU_10, 1,
    textures [0x1, 0xD, 0x7, 0x5], 16,
    kinds [0x280, 0x26A, 0x26A, 0x262, 0x26D], 5,
    [0x235C0, 0xFFFF4000, 0x27900, 0x20000]
);
menu!(
    MENU_11, 1,
    textures [0xA, 0x8, 0x5, 0x19], 16,
    kinds [0x26D, 0x266, 0x263, 0x28B, 0x28C, 0x28D, 0x28E], 7,
    [0x235C0, 0xFFFF4000, 0x27900, 0x20000]
);
menu!(
    MENU_12, 1,
    textures [0xB, 0xE, 0x10, 0x5], 16,
    kinds [0x267, 0x26B, 0x270, 0x26D], 4,
    [0x214C0, 0xFFFF4000, 0x36F00, 0x20000]
);
menu!(
    MENU_13, 1,
    textures [0xF, 0xC, 0xD, 0x7, 0x7, 0x13, 0x12], 16,
    kinds [0x26E, 0x26A, 0x27B, 0x262, 0x279, 0x277, 0x27F], 5,
    [0x235C0, 0xFFFF4000, 0x27900, 0x20000]
);
menu!(
    MENU_14, 1,
    textures [0xF, 0xC, 0xD, 0x7, 0x7, 0x13, 0x12], 16,
    kinds [0x26E, 0x26A, 0x27B, 0x262, 0x279, 0x277], 5,
    [0x235C0, 0xFFFF4000, 0x27900, 0x20000]
);

/// By menu number; `None` for the menus written by hand.
pub const PLAIN: [Option<&[Op]>; 21] = [
    Some(MENU_00),
    Some(MENU_01),
    Some(MENU_02),
    Some(MENU_03),
    Some(MENU_04),
    Some(MENU_05),
    Some(MENU_06),
    Some(MENU_07),
    Some(MENU_08),
    Some(MENU_09),
    Some(MENU_0A),
    None,
    None,
    None,
    None,
    Some(MENU_0F),
    Some(MENU_10),
    Some(MENU_11),
    Some(MENU_12),
    Some(MENU_13),
    Some(MENU_14),
];
