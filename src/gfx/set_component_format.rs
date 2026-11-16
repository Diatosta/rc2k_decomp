//! Describes the format of the game's own component pixels: the format at `component_format` gets
//! the masks 0xFF, 0xFF00, 0xFF0000 and 0xFF000000 and is described by `gfx::describe_format`.

use crate::errors::Refused;
use crate::gfx::describe_format::describe_format;
use crate::gfx::formats::Formats;

const MASKS: [u32; 4] = rc2k_core::gfx::pixels::COMPONENT_MASKS;

pub fn set_component_format(formats: &Formats) -> Result<(), Refused> {
    let format = &formats.component_format;
    for (n, mask) in MASKS.into_iter().enumerate() {
        format[1 + n].set(mask);
    }
    describe_format(formats, format);
    Ok(())
}
