//! Pictures loaded: which reader `gfx::read_picture_file` takes, and the overlay
//! `gfx::load_picture_with_overlay` lays over a picture.

use crate::Invalid;

/// The reader a picture path goes to after its ending was swapped when the file is missing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reader {
    /// `gfx::read_png`, then `gfx::read_pcx` when that fails.
    PngThenPcx,
    /// `gfx::read_pcx` of the path.
    Pcx,
    /// Nothing read, success.
    Nothing,
    /// `gfx::read_pcx` of `files\d\gfx\chatzone.pcx`.
    Chatzone,
}

/// The reader, by the path's last four characters and the setting (0 in the installs seen):
/// with 0 every path goes to the PNG reader first; else a `.png` is not read (2: success) or
/// replaced by the chat zone's picture, and other paths go to PCX.
pub fn reader(ending: [u8; 4], setting: u32) -> Reader {
    match setting {
        0 => Reader::PngThenPcx,
        _ if &ending != b".png" => Reader::Pcx,
        2 => Reader::Nothing,
        _ => Reader::Chatzone,
    }
}

/// The path's last four characters (a `.pcx` goes to the PCX reader straight away, before
/// the swap). Refused: a path of fewer than four.
pub fn ending(path: &[u8]) -> Result<[u8; 4], Invalid> {
    path.len()
        .checked_sub(4)
        .map(|n| path[n..].try_into().unwrap())
        .ok_or(Invalid("a picture path of fewer than four characters"))
}

/// The see-through colour of an overlay.
pub(crate) const MAGENTA: u32 = 0xFF00FF;

/// Every pixel of the overlay but magenta over the picture: `width` x `height`, the
/// overlay's rows `from_stride` bytes apart, the picture's `to_stride`. Refused: no width
/// or height, or rows past either buffer.
pub fn lay_over(
    to: &mut [u8],
    from: &[u8],
    width: usize,
    height: usize,
    from_stride: usize,
    to_stride: usize,
) -> Result<(), Invalid> {
    if width == 0 || height == 0 {
        return Err(Invalid("an overlay of no width or height"));
    }
    let last = |stride: usize| (height - 1) * stride + width * 4;
    if from.len() < last(from_stride) || to.len() < last(to_stride) {
        return Err(Invalid("an overlay past its buffer or the picture's"));
    }
    for y in 0..height {
        for x in 0..width {
            let f = y * from_stride + x * 4;
            let pixel = u32::from_le_bytes(from[f..f + 4].try_into().unwrap());
            if pixel != MAGENTA {
                let t = y * to_stride + x * 4;
                to[t..t + 4].copy_from_slice(&pixel.to_le_bytes());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readers_and_overlays() {
        assert_eq!(reader(*b".png", 0), Reader::PngThenPcx);
        assert_eq!(reader(*b".png", 2), Reader::Nothing);
        assert_eq!(reader(*b".png", 1), Reader::Chatzone);
        assert_eq!(reader(*b".tga", 1), Reader::Pcx);
        assert_eq!(ending(b"a.pcx").unwrap(), *b".pcx");
        assert!(ending(b"pcx").is_err());
        let mut to = vec![0u8; 16];
        let from: Vec<u8> = [1u32, MAGENTA, 3, 4]
            .iter()
            .flat_map(|p| p.to_le_bytes())
            .collect();
        lay_over(&mut to, &from, 2, 2, 8, 8).unwrap();
        assert_eq!(to[..8], [1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(to[8..], [3, 0, 0, 0, 4, 0, 0, 0]);
        assert!(lay_over(&mut to, &from, 3, 2, 8, 8).is_err());
    }
}
