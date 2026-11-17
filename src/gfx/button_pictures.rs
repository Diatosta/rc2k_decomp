//! The pictures of a controller's buttons (`assets/glyphs`, Kenney's input prompts, CC0): white
//! outlines with soft edges, held as one grey atlas for each family whose pixels are the
//! pictures' alpha. `draw_glyph` blends a picture, scaled to a height, over the screen's pixels
//! (16 and 32 bits). It writes to the canvas on both renderers: the GPU path draws the quads
//! noted so far onto the canvas before a canvas write, so the order stays right, and the
//! pictures lie over everything drawn before them. Where the GPU's own picture shows through (a
//! canvas pixel nothing has written) a pixel is written white with the picture's alpha in the
//! pixel's top byte, which the renderers' overlay blends with what is under it.

use rc2k_core::input::icons::{Family, Icon, parse_index};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// The side of a picture in the atlas (the pack's 128 px size: pictures are drawn at 20 to
/// 40 px, so the scaling down is by a factor of three to six and has the detail to be smooth).
const CELL: usize = 128;

/// A family's atlas: the alpha of each picture, `CELL` x `CELL` bytes, by picture.
struct Atlas {
    pictures: HashMap<Icon, Vec<u8>>,
}

fn built_in(family: Family) -> (&'static [u8], &'static str) {
    match family {
        Family::Xbox => (
            include_bytes!("../../assets/glyphs/xbox.png"),
            include_str!("../../assets/glyphs/xbox.idx"),
        ),
        Family::PlayStation => (
            include_bytes!("../../assets/glyphs/playstation.png"),
            include_str!("../../assets/glyphs/playstation.idx"),
        ),
        Family::SteamDeck => (
            include_bytes!("../../assets/glyphs/steamdeck.png"),
            include_str!("../../assets/glyphs/steamdeck.idx"),
        ),
    }
}

/// An 8-bit grey PNG that is not interlaced: (width, height, pixels).
fn decode_grey(file: &[u8]) -> Option<(usize, usize, Vec<u8>)> {
    if file.get(..8)? != rc2k_core::gfx::png::SIGNATURE {
        return None;
    }
    let be = |at: usize| Some(u32::from_be_bytes(file.get(at..at + 4)?.try_into().ok()?));
    let (mut at, mut size, mut data) = (8usize, None, Vec::new());
    while at + 8 <= file.len() {
        let length = be(at)? as usize;
        let body = file.get(at + 8..at + 8 + length)?;
        match &file[at + 4..at + 8] {
            b"IHDR" => {
                // 8 bits, grey, not interlaced
                if body.get(8..13)? != [8, 0, 0, 0, 0] {
                    return None;
                }
                size = Some((be(at + 8)? as usize, be(at + 12)? as usize));
            }
            b"IDAT" => data.extend_from_slice(body),
            b"IEND" => break,
            _ => {}
        }
        at += 12 + length;
    }
    let (width, height) = size?;
    let mut area = miniz_oxide::inflate::decompress_to_vec_zlib(&data).ok()?;
    if area.len() < (width + 1) * height {
        return None;
    }
    rc2k_core::gfx::png::unfilter(&mut area, width, height as u32, 1, &|_| 0).ok()?;
    area.truncate(width * height);
    Some((width, height, area))
}

fn load(family: Family) -> Option<Atlas> {
    let (file, index) = built_in(family);
    let (width, height, pixels) = decode_grey(file)?;
    let mut pictures = HashMap::new();
    for (icon, [x, y, w, h]) in parse_index(index) {
        let (x, y, w, h) = (x as usize, y as usize, w as usize, h as usize);
        if (w, h) != (CELL, CELL) || x + w > width || y + h > height {
            return None;
        }
        let mut cell = Vec::with_capacity(CELL * CELL);
        for row in y..y + h {
            cell.extend_from_slice(&pixels[row * width + x..row * width + x + w]);
        }
        pictures.insert(icon, cell);
    }
    Some(Atlas { pictures })
}

fn atlas(family: Family) -> Option<&'static Atlas> {
    static ATLASES: [OnceLock<Option<Atlas>>; 3] = [const { OnceLock::new() }; 3];
    let place = Family::ALL.iter().position(|&f| f == family)?;
    ATLASES[place].get_or_init(|| load(family)).as_ref()
}

/// Whether the family has a picture for `icon`.
pub fn has(family: Family, icon: Icon) -> bool {
    atlas(family).is_some_and(|a| a.pictures.contains_key(&icon))
}

/// The rows of picture `icon`'s cell its outline covers, as fractions of the cell's side: (the
/// first, the last + 1). Pictures leave a margin around their outline, so a picture drawn to
/// match a text's height is scaled by its outline, not by its cell.
pub fn outline_rows(family: Family, icon: Icon) -> Option<(f32, f32)> {
    let cell = atlas(family)?.pictures.get(&icon)?;
    let inked = |row: &[u8]| row.iter().any(|&a| a >= 0x40);
    let first = cell.chunks(CELL).position(inked)?;
    let last = cell.chunks(CELL).rposition(inked)?;
    Some((first as f32 / CELL as f32, (last + 1) as f32 / CELL as f32))
}

/// The atlas pixels along one axis that output pixel `n` of `size` covers, with how much of
/// each it covers (the weights add up to 1): the area filter, partial pixels at the ends
/// counting in proportion.
fn coverage(n: usize, size: usize) -> impl Iterator<Item = (usize, f32)> {
    let step = CELL as f32 / size as f32;
    let (from, to) = (n as f32 * step, (n + 1) as f32 * step);
    (from as usize..(to.ceil() as usize).min(CELL)).map(move |at| {
        let overlap = to.min(at as f32 + 1.0) - from.max(at as f32);
        (at, overlap.max(0.0) / step)
    })
}

/// A picture's alpha scaled to `size` x `size`: each pixel the mean of the atlas pixels it
/// covers, weighted by how much of each it covers (a box filter at any scale, so the edges of a
/// circle keep their soft, graded alpha rather than stepping).
pub fn scaled(cell: &[u8], size: usize) -> Vec<u8> {
    // across, then down
    let mut across = vec![0f32; size * CELL];
    for row in 0..CELL {
        for x in 0..size {
            across[row * size + x] = coverage(x, size)
                .map(|(at, weight)| cell[row * CELL + at] as f32 * weight)
                .sum();
        }
    }
    let mut out = vec![0u8; size * size];
    for y in 0..size {
        for x in 0..size {
            let value: f32 = coverage(y, size)
                .map(|(at, weight)| across[at * size + x] * weight)
                .sum();
            out[y * size + x] = (value + 0.5).min(255.0) as u8;
        }
    }
    out
}

/// A scaled picture, kept.
fn scaled_picture(family: Family, icon: Icon, size: usize) -> Option<Arc<Vec<u8>>> {
    type Key = (Family, Icon, usize);
    static KEPT: Mutex<Option<HashMap<Key, Arc<Vec<u8>>>>> = Mutex::new(None);
    let mut kept = KEPT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let kept = kept.get_or_insert_with(HashMap::new);
    if let Some(done) = kept.get(&(family, icon, size)) {
        return Some(done.clone());
    }
    let cell = atlas(family)?.pictures.get(&icon)?;
    let done = Arc::new(scaled(cell, size));
    kept.insert((family, icon, size), done.clone());
    Some(done)
}

/// White blended over `dst` (a channel) with `alpha`.
fn lighten(dst: u32, alpha: u32) -> u32 {
    dst + (255 - dst) * alpha / 255
}

/// A 32-bit pixel (0x00RRGGBB) with white blended over it.
pub fn blend_32(pixel: u32, alpha: u8) -> u32 {
    let a = alpha as u32;
    let channel = |shift: u32| lighten(pixel >> shift & 0xFF, a) << shift;
    channel(16) | channel(8) | channel(0)
}

/// A 16-bit pixel (5-6-5) with white blended over it.
pub fn blend_16(pixel: u16, alpha: u8) -> u16 {
    let a = alpha as u32;
    let p = pixel as u32;
    // each channel widened to 8 bits, blended, narrowed again
    let (r, g, b) = (p >> 11 & 0x1F, p >> 5 & 0x3F, p & 0x1F);
    let (r, g, b) = (r << 3 | r >> 2, g << 2 | g >> 4, b << 3 | b >> 2);
    let (r, g, b) = (lighten(r, a), lighten(g, a), lighten(b, a));
    ((r >> 3) << 11 | (g >> 2) << 5 | b >> 3) as u16
}

/// Picture `icon` of `family` drawn with its top left at (`x`, `y`), `height` pixels high and as
/// wide (a picture's cell is square; the outline fills most of it), over the surface the game
/// draws on, clipped to the screen. False when the picture or a surface is missing, or the
/// surface is neither 16 nor 32 bits. The surface must be locked.
pub fn draw_glyph(
    o: crate::owners::Owners<'static>,
    family: Family,
    icon: Icon,
    x: i32,
    y: i32,
    height: i32,
) -> bool {
    if height <= 0 {
        return false;
    }
    let Some(picture) = scaled_picture(family, icon, height as usize) else {
        return false;
    };
    let surface = crate::gfx::surface::surface(o.gfx);
    let bytes = match surface.bits_per_pixel {
        16 => 2,
        32 => 4,
        _ => return false,
    };
    let Ok(rows) = crate::gfx::rows::Rows::surface(o.gfx) else {
        return false;
    };
    let clip = crate::gfx::surface::clip(o.gfx);
    let size = height as usize;
    let (left, right) = (x.max(clip.left), (x + height - 1).min(clip.right));
    if left > right {
        return false;
    }
    for row in 0..height {
        let py = y + row;
        if py < clip.top || py > clip.bottom {
            continue;
        }
        let alpha = &picture[row as usize * size..(row as usize + 1) * size];
        let (from, to) = ((left - x) as usize, (right - x) as usize);
        if alpha[from..=to].iter().all(|&a| a == 0) {
            continue;
        }
        let at = (left * bytes) as u32;
        let len = (to - from + 1) * bytes as usize;
        let Ok(mut pixels) = rows.read(py as u32, at, len, "a button picture's row") else {
            return false;
        };
        for (pixel, &a) in pixels
            .chunks_exact_mut(bytes as usize)
            .zip(&alpha[from..=to])
        {
            if a == 0 {
                continue;
            }
            if bytes == 4 {
                let p = u32::from_le_bytes([pixel[0], pixel[1], pixel[2], pixel[3]]);
                // Under a pixel nothing has written the GPU's own picture shows, which the CPU
                // cannot blend with: the pixel is white carrying the alpha, for the overlay to
                // blend (a fully covered pixel has the top byte 0, as every opaque one).
                if p == rc2k_core::gfx::atlas::UNWRITTEN_PIXEL {
                    let top = if a == 255 { 0 } else { u32::from(a) << 24 };
                    pixel.copy_from_slice(&(0x00FF_FFFF | top).to_le_bytes());
                    continue;
                }
                // White over a translucent white of an earlier picture: the alphas combine.
                if (1..255).contains(&(p >> 24)) {
                    let (under, over) = (p >> 24, u32::from(a));
                    let both = over + under * (255 - over) / 255;
                    let top = if both >= 255 { 0 } else { both << 24 };
                    pixel.copy_from_slice(&(0x00FF_FFFF | top).to_le_bytes());
                    continue;
                }
                pixel.copy_from_slice(&blend_32(p, a).to_le_bytes());
            } else {
                let p = u16::from_le_bytes([pixel[0], pixel[1]]);
                pixel.copy_from_slice(&blend_16(p, a).to_le_bytes());
            }
        }
        if rows
            .write(py as u32, at, &pixels, "a button picture's row")
            .is_err()
        {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_family_has_the_pictures_its_index_names() {
        for family in Family::ALL {
            let atlas = atlas(family).expect("the atlas decodes");
            let (_, index) = built_in(family);
            assert_eq!(atlas.pictures.len(), parse_index(index).len());
            for icon in [
                Icon::South,
                Icon::East,
                Icon::West,
                Icon::North,
                Icon::L1,
                Icon::Start,
            ] {
                assert!(has(family, icon), "{family:?} {icon:?}");
            }
            // a picture is not empty
            assert!(atlas.pictures[&Icon::South].iter().any(|&a| a > 200));
        }
        // the Steam Deck's stick presses are the Xbox's pictures
        assert!(has(Family::SteamDeck, Icon::L3));
        assert!(!has(Family::Xbox, Icon::Touchpad));
        assert!(has(Family::PlayStation, Icon::Touchpad));
    }

    #[test]
    fn scaling_keeps_the_mean() {
        let cell = vec![255u8; CELL * CELL];
        assert!(scaled(&cell, 32).iter().all(|&a| a == 255));
        assert_eq!(scaled(&cell, 24).len(), 24 * 24);
        let mut half = vec![0u8; CELL * CELL];
        for row in 0..CELL {
            for col in CELL / 2..CELL {
                half[row * CELL + col] = 255;
            }
        }
        let small = scaled(&half, 16);
        assert_eq!(small[0], 0);
        assert_eq!(small[15], 255);
        // a size that does not divide the cell: the middle pixel is half covered
        let odd = scaled(&half, 25);
        assert!((127..=128).contains(&odd[12]));
        assert_eq!(odd[11], 0);
        assert_eq!(odd[13], 255);
    }

    #[test]
    fn blending_white_lightens() {
        assert_eq!(blend_32(0x0000_0000, 255), 0x00FF_FFFF);
        assert_eq!(blend_32(0x0012_3456, 0), 0x0012_3456);
        assert_eq!(blend_16(0, 255), 0xFFFF);
        assert_eq!(blend_16(0x1234, 0), 0x1234);
    }
}
