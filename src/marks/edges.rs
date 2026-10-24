//! `init_marks`' step 1, its second half (`sub_4C0F2E`; IDA, 2026-10-17): the kinds'
//! records scaled once, the kind's edge picture (`files\m\edgeNN.pcx`, 256 x 128) packed to
//! a byte a pixel, the shape tilt of the kind, then the masks and the two item kinds.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::math::x87;

pub const DONE_RVA: usize = rva(0x8BC3D6); // g_marksEdgesDone: 0, or where to go on
pub const RESUME_RVA: usize = rva(0x4C0FFD); // loc_4C0FFD
pub const BOXES_RVA: usize = rva(0x811BEB); // g_marksKindBoxes: 0x22 of 0x30
pub const KIND_FLOATS_RVA: usize = rva(0x81224B); // g_marksKindFloats: 0x22
const KINDS: u32 = 0x22;
const SCALE_RVA: [usize; 3] = [rva(0x8BC3DA), rva(0x8BC3DE), rva(0x8BC3E2)];
const SETTING: u32 = 0x2708;
const SETTING_VALUE: u32 = 0xFFFF_7B2E;
pub const PATH_RVA: usize = rva(0x8BC3E6); // "files\m\edge00.pcx"
const DIGITS: u32 = 12; // the two digits' place in the path
pub const EDGE_RVA: usize = rva(0x809BC3); // g_marksEdgeMask: 0x8000 bytes
pub const EDGE_LEN: u32 = 0x8000;
const TILT_SCALE_RVA: usize = rva(0x8BC3F9); // flt_8BC3F9
pub const TILT_RVA: usize = rva(0x811BC3); // g_marksShapeTilt
const MOST_KIND: i32 = 0x16;

/// The kind, held to 0 .. 0x16.
fn clamped(kind: u32) -> u32 {
    (kind as i32).clamp(0, MOST_KIND) as u32
}

/// `marks::load_edges` (0x4C0F2E): once (`g_marksEdgesDone`, then `resume`) each kind's
/// record (`x87::scale_record` by 65536 / 1000) and its float less from 90; the settings'
/// +0x2708 0xFFFF7B2E; the kind (held to 0 .. 0x16) written as two digits into the edge
/// picture's path; that read (`gfx::read_picture_file`, at 256 x 128, the picture's sizes
/// kept round it); its pixels packed (the four bytes or'd, inverted) into
/// `g_marksEdgeMask`; the shape tilt the kind's float times 0.0175; then `make_masks`,
/// `set_up_hexagons`, `set_up_squares` (and `nullsub_17`). Refuses a mark neither 0 nor
/// `resume`, before writing; a picture that is not read (the original then counts
/// 0x1FFFFFFF down and leaves with its stack one off), after the path is written.
pub unsafe fn load_edges(resume: u32) -> Result<(), Refused> {
    use crate::gfx::picture::{HEIGHT_RVA, PICTURE_RVA, ROW_BYTES_RVA, WIDTH_RVA};
    unsafe {
        let done: u32 = game_read(DONE_RVA);
        if done != 0 && done != resume {
            return Err(BadInput("the edges' mark is neither 0 nor loc_4C0FFD").into());
        }
        if done == 0 {
            let boxes = game_addr(BOXES_RVA) as u32;
            let floats = game_addr(KIND_FLOATS_RVA) as u32;
            let scale: [u32; 3] = SCALE_RVA.map(|r| game_read(r));
            for k in 0..KINDS {
                x87::scale_record((boxes + 0x30 * k) as *mut u32, scale[0], scale[1]);
                let f = (floats + 4 * k) as *mut u32;
                f.write_unaligned(x87::sub(scale[2], f.read_unaligned()));
            }
            game_write(DONE_RVA, resume);
        }
        let settings: u32 = game_read(crate::marks::lines::SETTINGS_RVA);
        ((settings + SETTING) as *mut u32).write_unaligned(SETTING_VALUE);
        let kind = clamped(game_read(crate::marks::kind_values::KIND_RVA));
        let path = game_addr(PATH_RVA) as u32;
        *((path + DIGITS) as *mut u8) = (kind / 10) as u8 + b'0';
        *((path + DIGITS + 1) as *mut u8) = (kind % 10) as u8 + b'0';
        let sizes: [u32; 3] = [
            game_read(WIDTH_RVA),
            game_read(HEIGHT_RVA),
            game_read(ROW_BYTES_RVA),
        ];
        game_write(WIDTH_RVA, 0x100u32);
        game_write(HEIGHT_RVA, 0x80u32);
        game_write(ROW_BYTES_RVA, 0xFFFF_FFFEu32);
        let read = crate::gfx::read_picture_file::read_picture_file(path as *mut u8);
        game_write(WIDTH_RVA, sizes[0]);
        game_write(HEIGHT_RVA, sizes[1]);
        game_write(ROW_BYTES_RVA, sizes[2]);
        if !read? {
            return Err(BadInput("the edge picture was not read").into());
        }
        let picture: u32 = game_read(PICTURE_RVA);
        let edge = game_addr(EDGE_RVA) as u32;
        for k in 0..EDGE_LEN {
            let p = std::slice::from_raw_parts((picture + 4 * k) as *const u8, 4);
            *((edge + k) as *mut u8) = !(p[0] | p[1] | p[2] | p[3]);
        }
        let float: u32 = game_read(KIND_FLOATS_RVA + 4 * kind as usize);
        game_write(TILT_RVA, x87::mul(float, game_read(TILT_SCALE_RVA)));
        crate::marks::masks::make_masks()?;
        crate::marks::shapes::set_up_hexagons(
            game_addr(crate::marks::shapes::HEXAGONS.resume) as u32
        )?;
        crate::marks::shapes::set_up_squares(
            game_addr(crate::marks::shapes::SQUARES.resume) as u32
        )?;
    }
    Ok(())
}

const KIND_AT: u32 = 0x26A8; // the settings' kind

/// `marks::load_kind_data` (0x4C1235), `init_marks`' step 1: `g_marksKind` the settings'
/// +0x26A8; `nullsub_1` (twice, between them), `load_edges`, `load_splines`. Refuses as
/// its steps do.
pub unsafe fn load_kind_data() -> Result<(), Refused> {
    unsafe {
        let settings: u32 = game_read(crate::marks::lines::SETTINGS_RVA);
        let kind = ((settings + KIND_AT) as *const u32).read_unaligned();
        game_write(crate::marks::kind_values::KIND_RVA, kind);
        load_edges(game_addr(RESUME_RVA) as u32)?;
        crate::marks::splines::load_splines(game_addr(crate::marks::splines::RESUME_RVA) as u32)?;
    }
    Ok(())
}
