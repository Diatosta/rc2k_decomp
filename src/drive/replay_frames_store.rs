//! The replay's frame variables: the first step, the cameras' words, the frame and tenth kept, the step's block and frame, the eight kept frames of the car blocks (`ReplayFrames.slots`, 0x3F80 bytes each) and the last frame `quit_question` keeps (`ReplayFrames.last_kept`, just before `g_carBlockStore`). Names are readings unless IDA's.

use crate::session::{PlainBytes, PlainDword};

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ReplayFrames {
    /// +0x0 (0x6EF7CC): `dword_6EF7CC`: set while the replay steps to a still (`physics::ease_view_turn`, `race::show_replay_still`).
    pub step_first: PlainDword,
    /// +0x4 (0x6EF7D0): `dword_6EF7D0`: the first car block's first dword (`drive::replay_cameras`).
    pub camera_word_0: PlainDword,
    /// +0x8 (0x6EF7D4): `dword_6EF7D4`: the first car block's second dword (`drive::replay_cameras`).
    pub camera_word_1: PlainDword,
    /// +0xc (0x6EF7D8): `dword_6EF7D8`: the replay's frame (`drive::keep_replay_frame`).
    pub frame: PlainDword,
    /// +0x10 (0x6EF7DC): `dword_6EF7DC`: the tenth within the frame (`drive::keep_replay_frame`, `drive::record_inputs`, `drive::flush_replay_if`).
    pub tenth: PlainDword,
    /// +0x14 (0x6EF7E0): `dword_6EF7E0`: the block the step is in (`drive::load_replay_frame`, `drive::play_replay_frame`, `race::show_replay_still`).
    pub step_block: PlainDword,
    /// +0x18 (0x6EF7E4): `dword_6EF7E4`: the frame the step is at (`drive::play_inputs`, `drive::play_replay_frame`).
    pub step_frame: PlainDword,
    /// +0x1c (0x6EF7E8): `unk_6EF7E8`: eight kept frames of 0x3F80 bytes (`drive::keep_replay_frame`).
    pub slots: PlainBytes<0x1FC00>,
    /// +0x1fc1c (0x70F3E8): `dword_70F3E8`: the last frame `drive::quit_question` kept (-1 from `race::stage_loading`).
    pub last_kept: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl ReplayFrames {
    pub const fn new() -> Self {
        use crate::cells::ZeroCells;
        use crate::session::{PlainBytes, with_runs};
        Self {
            slots: PlainBytes::new(with_runs::<130048>(&[
                (48, b"\x80\x00\x00\x00@"),
                (664, b"\xff\xff\xff\xff"),
                (4112, b"\x80\x00\x00\x00@"),
                (4728, b"\xff\xff\xff\xff"),
                (8176, b"\x80\x00\x00\x00@"),
                (8792, b"\xff\xff\xff\xff"),
                (12240, b"\x80\x00\x00\x00@"),
                (12856, b"\xff\xff\xff\xff"),
                (16304, b"\x80\x00\x00\x00@"),
                (16920, b"\xff\xff\xff\xff"),
                (20368, b"\x80\x00\x00\x00@"),
                (20984, b"\xff\xff\xff\xff"),
                (24432, b"\x80\x00\x00\x00@"),
                (25048, b"\xff\xff\xff\xff"),
                (28496, b"\x80\x00\x00\x00@"),
                (29112, b"\xff\xff\xff\xff"),
                (32560, b"\x80\x00\x00\x00@"),
                (33176, b"\xff\xff\xff\xff"),
                (36624, b"\x80\x00\x00\x00@"),
                (37240, b"\xff\xff\xff\xff"),
                (40688, b"\x80\x00\x00\x00@"),
                (41304, b"\xff\xff\xff\xff"),
                (44752, b"\x80\x00\x00\x00@"),
                (45368, b"\xff\xff\xff\xff"),
                (48816, b"\x80\x00\x00\x00@"),
                (49432, b"\xff\xff\xff\xff"),
                (52880, b"\x80\x00\x00\x00@"),
                (53496, b"\xff\xff\xff\xff"),
                (56944, b"\x80\x00\x00\x00@"),
                (57560, b"\xff\xff\xff\xff"),
                (61008, b"\x80\x00\x00\x00@"),
                (61624, b"\xff\xff\xff\xff"),
                (65072, b"\x80\x00\x00\x00@"),
                (65688, b"\xff\xff\xff\xff"),
                (69136, b"\x80\x00\x00\x00@"),
                (69752, b"\xff\xff\xff\xff"),
                (73200, b"\x80\x00\x00\x00@"),
                (73816, b"\xff\xff\xff\xff"),
                (77264, b"\x80\x00\x00\x00@"),
                (77880, b"\xff\xff\xff\xff"),
                (81328, b"\x80\x00\x00\x00@"),
                (81944, b"\xff\xff\xff\xff"),
                (85392, b"\x80\x00\x00\x00@"),
                (86008, b"\xff\xff\xff\xff"),
                (89456, b"\x80\x00\x00\x00@"),
                (90072, b"\xff\xff\xff\xff"),
                (93520, b"\x80\x00\x00\x00@"),
                (94136, b"\xff\xff\xff\xff"),
                (97584, b"\x80\x00\x00\x00@"),
                (98200, b"\xff\xff\xff\xff"),
                (101648, b"\x80\x00\x00\x00@"),
                (102264, b"\xff\xff\xff\xff"),
                (105712, b"\x80\x00\x00\x00@"),
                (106328, b"\xff\xff\xff\xff"),
                (109776, b"\x80\x00\x00\x00@"),
                (110392, b"\xff\xff\xff\xff"),
                (113840, b"\x80\x00\x00\x00@"),
                (114456, b"\xff\xff\xff\xff"),
                (117904, b"\x80\x00\x00\x00@"),
                (118520, b"\xff\xff\xff\xff"),
                (121968, b"\x80\x00\x00\x00@"),
                (122584, b"\xff\xff\xff\xff"),
                (126032, b"\x80\x00\x00\x00@"),
                (126648, b"\xff\xff\xff\xff"),
            ])),
            ..ZeroCells::ZERO
        }
    }
}
