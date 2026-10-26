//! The race's variables before the draw state (IDA 0x6235E8 .. 0x623F80; phase 7, typed): the
//! race's settings as the race code reads them (stage, direction, car kinds, tyres, run kind,
//! split, settings words; `race::settings_copy` fills them), the checkpoints and the best
//! line's work, the loading bar, the mud texture's state, and the flat quads' corners,
//! colours and vertices. `#[repr(C)]` with the image's layout, over the race block's bytes
//! (`RACE_VARS`), as `CAR`. Names are readings unless IDA's.

use crate::session::{Dword, Float};

#[repr(C)]
pub struct RaceVars {
    /// +0x0 (0x6235E8): a scene switch (`race::show_replay_still`; read by `physics::car_lighting`, `race::skid_laying`).
    pub scene_switch: Dword,
    /// +0x4 (0x6235EC): the held count (`drive::hold_count_tick`; `drive::pause_until_key` sets 0x19).
    pub hold_count: Dword,
    /// +0x8 (0x6235F0): `drive::pause_until_key`'s wait (0x1E, counted down).
    pub pause_wait: Dword,
    /// +0xc (0x6235F4): put back into `dword_606294` at the race's end (`race::race_end`).
    pub kept_scale: Dword,
    /// +0x10 (0x6235F8): the race's tick count (`drive::race_tick`).
    pub race_ticks: Dword,
    /// +0x14 (0x6235FC): each car's tyres (the players' `settings.cars[n].tyre`); the car's gravel
    /// texture switch (`scene::load_texture_picture` reads eight).
    pub car_tyres: [Dword; 8],
    /// +0x34 (0x62361C): (`scene::list_object_faces`).
    pub face_mark: Dword,
    /// +0x38 (0x623620): `g_faceNumber` ("objfaced").
    pub face_number: Dword,
    /// +0x3c (0x623624): not named.
    pub unknown_3c: [Dword; 4],
    /// +0x4c (0x623634): the loading bar's span, from and to (`scene::step_loading_bar`; `race::loadlevel`'s object span).
    pub bar_span: [Dword; 2],
    /// +0x54 (0x62363C): `g_lineFaceTotal` ("numblo", `race::best_line`).
    pub line_face_total: Dword,
    /// +0x58 (0x623640): `g_checkpointCount`.
    pub checkpoint_count: Dword,
    /// +0x5c (0x623644): `g_lastCheckpoint` ("numsortedm1").
    pub last_checkpoint: Dword,
    /// +0x60 (0x623648): not named.
    pub unknown_60: [Dword; 3],
    /// +0x6c (0x623654): `g_lineFaceKind`.
    pub line_face_kind: Dword,
    /// +0x70 (0x623658): a point's x, z (`race::best_line`).
    pub line_from: [Dword; 2],
    /// +0x78 (0x623660): `g_replayCameraCount1`, `2`.
    pub replay_camera_counts: [Dword; 2],
    /// +0x80 (0x623668): the nearest distance, then the one (`race::best_line`).
    pub line_nearest: [Dword; 2],
    /// +0x88 (0x623670): `g_surfaceShares`.
    pub surface_shares: [Float; 16],
    /// +0xc8 (0x6236B0): `g_surfaceWeight`, a whole number.
    pub surface_weight: Dword,
    /// +0xcc (0x6236B4): `g_mapAnchorCount`.
    pub map_anchor_count: Dword,
    /// +0xd0 (0x6236B8): the map: the stretch's two anchors and what is made of them
    /// (`race::best_line`, its offsets 0xB8 .. 0xF8 and one dword more).
    pub map_work: [Dword; 18],
    /// +0x118 (0x623700): the loading bar's share loaded, 0 .. 100.
    pub bar_share: Dword,
    /// +0x11c (0x623704): the loading bar's function (0: none).
    pub bar_function: Dword,
    /// +0x120 (0x623708): not named.
    pub unknown_120: [Dword; 82],
    /// +0x268 (0x623850): `g_stageNumber` (`settings.stage`).
    pub stage_number: Dword,
    /// +0x26c (0x623854): the stage's direction (`settings.direction`).
    pub direction: Dword,
    /// +0x270 (0x623858): `g_raceCarKinds`: [0] the current (`settings.car_kind_current`), [1] the first (`car_kind_first`), [2 ..] `[shauncarnum3]` ...
    pub race_car_kinds: [Dword; 8],
    /// +0x290 (0x623878): each car's colours word (`net::set_up_race`; the stored blocks' gearbox kind; the players' `settings.cars[n].record_30`).
    pub car_colours: [Dword; 8],
    /// +0x2b0 (0x623898): the textures' mip setting (`scene::set_filters`, `scene::try_allocate_textures`; from `dword_4F50D0`).
    pub mip_setting: Dword,
    /// +0x2b4 (0x62389C): the race's run kind (`settings.run_kind`; 2: the cars by place; splits, views, hidden kinds).
    pub run_kind: Dword,
    /// +0x2b8 (0x6238A0): not named.
    pub unknown_2b8: Dword,
    /// +0x2bc (0x6238A4): `settings.kept`: the screen split / players (`race::frame`, `hud::overlay_pass`, the gear choice).
    pub split: Dword,
    /// +0x2c0 (0x6238A8): `settings.setting_0` .. `4` ([1] the shine and the front end's set-up, [4] all cars' lights).
    pub settings_words: [Dword; 5],
    /// +0x2d4 (0x6238BC): `g_rivals` (`settings.rivals`: the standings summary's block id).
    pub rivals: Dword,
    /// +0x2d8 (0x6238C0): `settings.no_replay`.
    pub no_replay: Dword,
    /// +0x2dc (0x6238C4): not named.
    pub unknown_2dc: [Dword; 2],
    /// +0x2e4 (0x6238CC): `results.race_level`.
    pub race_level: Dword,
    /// +0x2e8 (0x6238D0): `results.race_stage`, less 1.
    pub race_stage: Dword,
    /// +0x2ec (0x6238D4): the cars' start numbers (`net::set_up_race`).
    pub start_numbers: [Dword; 8],
    /// +0x30c (0x6238F4): "files\b\mudfile.0xx" (`race::mud`; the digits at +0x11).
    pub mud_path: crate::session::Bytes<20>,
    /// +0x320 (0x623908): not named.
    pub unknown_320: Dword,
    /// +0x324 (0x62390C): [0] turned into [1] (`race::mud_texture_turn`).
    pub mud_turn_a: [Dword; 2],
    /// +0x32c (0x623914): not named.
    pub unknown_32c: Dword,
    /// +0x330 (0x623918): the same.
    pub mud_turn_b: [Dword; 2],
    /// +0x338 (0x623920): (`race::update_mud_texture`).
    pub mud_throws: Dword,
    /// +0x33c (0x623924): (`race::update_mud_texture`).
    pub mud_splashes: Dword,
    /// +0x340 (0x623928): not named.
    pub unknown_340: [Dword; 2],
    /// +0x348 (0x623930): `g_mudColours`.
    pub mud_colours: [Dword; 256],
    /// +0x748 (0x623D30): (`race::throw_mud_texel`).
    pub mud_extra_sum: Dword,
    /// +0x74c (0x623D34): (`race::mud_texture_turn`).
    pub mud_packed: Dword,
    /// +0x750 (0x623D38): (`race::splash_mud`).
    pub mud_column: Dword,
    /// +0x754 (0x623D3C): (`race::splash_mud`).
    pub mud_row: Dword,
    /// +0x758 (0x623D40): not named.
    pub unknown_758: [Dword; 4],
    /// +0x768 (0x623D50): `g_mud`: the mud file's memory.
    pub mud: Dword,
    /// +0x76c (0x623D54): the texture locked (`race::update_mud_texture`).
    pub mud_locked: Dword,
    /// +0x770 (0x623D58): `g_mudFrames` (the mud state's first of 18 dwords).
    pub mud_frames: Dword,
    /// +0x774 (0x623D5C): `g_mudTexture`: the mud texture worked on.
    pub mud_texture: Dword,
    /// +0x778 (0x623D60): `g_mudThrows`: each texture's thrown mud.
    pub mud_thrown_in: [Float; 4],
    /// +0x788 (0x623D70): `g_mudSplashes`: each texture's splashes.
    pub mud_splashes_in: [Float; 4],
    /// +0x798 (0x623D80): `g_mudThrownCounts`.
    pub mud_thrown_counts: [Dword; 4],
    /// +0x7a8 (0x623D90): `g_mudSplashCounts`.
    pub mud_splash_counts: [Dword; 4],
    /// +0x7b8 (0x623DA0): not named.
    pub unknown_7b8: [Dword; 2],
    /// +0x7c0 (0x623DA8): [0] the drive's, [1] the slip's (`physics::turn_wheel_angles`).
    pub wheel_turn_a: [Float; 2],
    /// +0x7c8 (0x623DB0): the same, the second.
    pub wheel_turn_b: [Float; 2],
    /// +0x7d0 (0x623DB8): (`scene::draw_marked_polygon`).
    pub polygon_mark: Dword,
    /// +0x7d4 (0x623DBC): (`scene::draw_polygon`, `race::skidmarks`).
    pub fade_uv: [Dword; 2],
    /// +0x7dc (0x623DC4): not named.
    pub unknown_7dc: [Dword; 2],
    /// +0x7e4 (0x623DCC): a factor of the view's scale (`scene::set_view_window`).
    pub scale_factor: Float,
    /// +0x7e8 (0x623DD0): (`scene::place_object_points`, `scene::set_object_material`).
    pub move_by: Dword,
    /// +0x7ec (0x623DD4): the same.
    pub move_fraction: Float,
    /// +0x7f0 (0x623DD8): a texture mapping (`race::course_faces`).
    pub mapping_a: [Dword; 2],
    /// +0x7f8 (0x623DE0): another.
    pub mapping_b: [Dword; 2],
    /// +0x800 (0x623DE8): the quad's colour and see-through (`scene::draw_flat_quad`, `scene::draw_tinted_quad`).
    pub quad_colour: [Dword; 2],
    /// +0x808 (0x623DF0): the quad's texture coordinates (`race::frame_flush`).
    pub quad_uv: [Dword; 8],
    /// +0x828 (0x623E10): [0] left, [1] right, [4] top, [5] bottom.
    pub quad_corners: [Dword; 8],
    /// +0x848 (0x623E30): not named.
    pub unknown_848: [Dword; 4],
    /// +0x858 (0x623E40): four vertices of 0x20 (`scene::draw_flat_quad`, `race::course_faces`).
    pub quad_vertices: [Dword; 32],
    /// +0x8d8 (0x623EC0): not named.
    pub unknown_8d8: [Dword; 48],
}

/// The piece: the race block's bytes at IDA 0x6235E8 seen as `RaceVars`. They stay there
/// while code reaches them by RVA from `race::store::asm_base()` and by IDA address
/// (`moved::place`): one copy for all three.
pub struct InRaceBlock;

impl core::ops::Deref for InRaceBlock {
    type Target = RaceVars;
    fn deref(&self) -> &RaceVars {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); every field is an atomic or a cell, as the block's bytes were shared.
        unsafe { &*(crate::pointers::host(piece_addr()) as *const RaceVars) }
    }
}

pub static RACE_VARS: InRaceBlock = InRaceBlock;

/// IDA 0x6235E8, the first byte of the piece.
pub const START: usize = 0x6235E8;
/// IDA 0x623F80, the end of the piece.
pub const END: usize = 0x623F80;

const _: () = assert!(END - START == core::mem::size_of::<RaceVars>());

/// Where IDA 0x6235E8 is now (in the race block).
pub fn piece_addr() -> u32 {
    crate::race::store::home(START) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_race_block() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(offset_of!(RaceVars, scene_switch), 0x6235E8 - START);
        assert_eq!(offset_of!(RaceVars, face_mark), 0x62361C - START);
        assert_eq!(offset_of!(RaceVars, line_face_kind), 0x623654 - START);
        assert_eq!(offset_of!(RaceVars, map_anchor_count), 0x6236B4 - START);
        assert_eq!(offset_of!(RaceVars, race_car_kinds), 0x623858 - START);
        assert_eq!(offset_of!(RaceVars, rivals), 0x6238BC - START);
        assert_eq!(offset_of!(RaceVars, mud_turn_a), 0x62390C - START);
        assert_eq!(offset_of!(RaceVars, mud_packed), 0x623D34 - START);
        assert_eq!(offset_of!(RaceVars, mud_texture), 0x623D5C - START);
        assert_eq!(offset_of!(RaceVars, wheel_turn_b), 0x623DB0 - START);
        assert_eq!(offset_of!(RaceVars, mapping_a), 0x623DD8 - START);
    }
}
