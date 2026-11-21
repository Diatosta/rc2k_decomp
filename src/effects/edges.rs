//! `init_marks`' step 1, its second half (`effects::load_edges`): the kinds' records scaled
//! once, the kind's edge picture (`files\m\edgeNN.pcx`, 256 x 128) packed to a byte a
//! pixel, the shape tilt of the kind, then the masks and the two item kinds.

use crate::effects::mask_store::Masks;
use crate::effects::owner::Effects;
use crate::effects::windscreen_drop_vars_store::WindscreenDrops;
use crate::errors::{Recover, Refused};
use crate::math::x87;
use crate::memory::session::read_bits;

const KINDS: u32 = 0x22;
const SETTING_VALUE: u32 = 0xFFFF_7B2E;
const DIGITS: u32 = 12; // the two digits' place in the path
const EDGE_LEN: u32 = 0x8000;
const MOST_KIND: i32 = 0x16;

/// The kind, held to 0 .. 0x16.
fn clamped(kind: u32) -> u32 {
    (kind as i32).clamp(0, MOST_KIND) as u32
}

/// `effects::load_edges`: once (then the done flag is set) each kind's record (`x87::scale_record`
/// by 65536 / 1000) and its float less from 90; the settings' `light_point_a_negated[0]`
/// 0xFFFF7B2E; the kind (held to 0 .. 0x16) written as two digits into the edge picture's path;
/// that read (`gfx::read_picture_file`, at 256 x 128, the picture's sizes kept round it); its
/// pixels packed (the four bytes or'd, inverted) into the edge mask; the shape tilt the kind's
/// float times 0.0175; then `make_masks`, `set_up_windscreen_drops`, `set_up_windscreen_flakes`. A
/// picture that is not read is a trouble the game goes back from (`Recover`), after the path is
/// written.
#[allow(
    clippy::too_many_arguments,
    reason = "the edges' inputs and the owners they are read from"
)]
pub(super) fn load_edges(
    race: &crate::race::owner::Race,
    masks: &Masks,
    drops: &WindscreenDrops,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    effects: &crate::effects::owner::Effects,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    if !masks.edge_settings.done.is_set() {
        let mut kind_boxes = masks.kind_boxes.borrow_mut();
        let scale: [u32; 3] = read_bits(&masks.edge_settings.scale);
        for k in 0..KINDS {
            let at = (0x30 * k) as usize;
            let record = core::array::from_fn(|i| {
                let word: [u8; 4] = kind_boxes.boxes[at + 4 * i..at + 4 * i + 4]
                    .try_into()
                    .expect("four bytes");
                u32::from_le_bytes(word)
            });
            let scaled = x87::scale_record_bits(record, scale[0], scale[1]);
            for (i, v) in scaled.into_iter().enumerate() {
                kind_boxes.boxes[at + 4 * i..at + 4 * i + 4].copy_from_slice(&v.to_le_bytes());
            }
            let f = &mut kind_boxes.kind_floats[k as usize];
            *f = x87::sub(scale[2], *f);
        }
        masks.edge_settings.done.set(true);
    }
    let marks = effects.settings()?;
    marks.light_point_a_negated[0].set(SETTING_VALUE);
    let kind = clamped(effects.kinds.marks_kind.get());
    masks
        .edge_settings
        .path
        .set(DIGITS as usize, (kind / 10) as u8 + b'0');
    masks
        .edge_settings
        .path
        .set(DIGITS as usize + 1, (kind % 10) as u8 + b'0');
    let sizes: [u32; 3] = [
        gfx.pictures.picture.width.get(),
        gfx.pictures.picture.height.get(),
        gfx.pictures.picture.row_bytes.get(),
    ];
    gfx.pictures.picture.width.set(0x100u32);
    gfx.pictures.picture.height.set(0x80u32);
    gfx.pictures.picture.row_bytes.set(0xFFFF_FFFEu32);
    crate::logfile::load_times::mark("edges: read picture");
    let read =
        crate::gfx::read_picture_file::read_picture_file(&masks.edge_settings.path, gfx, system);
    gfx.pictures.picture.width.set(sizes[0]);
    gfx.pictures.picture.height.set(sizes[1]);
    gfx.pictures.picture.row_bytes.set(sizes[2]);
    if !read? {
        return Err(Recover("a picture the stage needs could not be read").into());
    }
    crate::logfile::load_times::mark("edges: mask");
    let pixels = crate::gfx::picture::read_bytes(
        &gfx.pictures.buffers,
        gfx.pictures.picture.buffer.get(),
        0,
        4 * EDGE_LEN as usize,
    )?;
    let mut edge_mask = masks.edge_mask.borrow_mut();
    for (k, p) in pixels.as_chunks::<4>().0.iter().enumerate() {
        edge_mask.mask[k] = !(p[0] | p[1] | p[2] | p[3]);
    }
    drop(edge_mask);
    let float = masks.kind_boxes.borrow().kind_floats[kind as usize];
    drops.tilt.borrow_mut().tilt = x87::mul(float, masks.edge_settings.tilt_scale.get());
    crate::logfile::load_times::mark("edges: make_masks");
    crate::effects::masks::make_masks(&race.course, race.faces(), masks, drops, scene, effects)?;
    crate::logfile::load_times::mark("edges: windscreen drops");
    crate::effects::shapes::set_up_windscreen_drops(drops, gfx, effects)?;
    crate::logfile::load_times::mark("edges: windscreen flakes");
    crate::effects::shapes::set_up_windscreen_flakes(
        &effects.pong,
        &effects.windscreen_flakes,
        effects,
    )?;
    Ok(())
}

/// `effects::load_kind_data`, `init_marks`' step 1: the kind taken from the settings'
/// `mark_car_file_number`; then `load_edges`, `load_splines`. Refuses as its steps do.
pub(super) fn load_kind_data(
    race: &crate::race::owner::Race,
    effects: &Effects,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let (masks, drops) = (&effects.masks, &effects.windscreen_drops);
    {
        let marks = effects.settings()?;
        let kind = marks.mark_car_file_number.get();
        effects.kinds.marks_kind.set(kind);
        crate::logfile::load_times::mark("edges: scale kinds");
        load_edges(race, masks, drops, scene, gfx, effects, system)?;
        crate::logfile::load_times::mark("splines");
        crate::effects::splines::load_splines(
            &race.course,
            race.faces(),
            &effects.paths,
            scene,
            effects,
        )?;
    }
    Ok(())
}
