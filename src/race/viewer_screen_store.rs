//! The replay viewer's screen (IDA 0x5C291C .. 0x5C2FE4; phase 7, typed): the cameras' text ids, the bar, buttons, marks and place rectangles (scratch ones written each frame), the quad scale floats, the picture switches, the replay patch switches kept, the frame-picture recording state (three dwords at odd addresses), the NET and display text strings between 0x5C2CF7 and 0x5C2F81 (left as unknown bytes), the view marks' state and the listener place and turn. Names are readings unless IDA's.

use crate::race::owner::race;
use crate::session::{Dword, Float};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ViewerPanelBlock {
    /// +0x0 (0x5C291C): `ViewerAndKinds.viewer_panel.camera_name_ids[0]` / `g_cameraNameIds`: 16-bit text ids of the camera names, read and written by `race::draw_viewer_panel`, `next_camera`, `previous_camera`; the port reaches them through another path, not by this field.
    pub camera_name_ids: [Dword; 24],
    /// +0x60 (0x5C297C): `ViewerAndKinds.viewer_panel.bar_frame_rect[0]`: rectangle (0,0,800,32) of the viewer bar's frame (`race::viewer_frame`).
    pub bar_frame_rect: [Dword; 4],
    /// +0x70 (0x5C298C): `ViewerAndKinds.viewer_panel.button_rect_a[0]`: first button rectangle (`race::viewer_frame`).
    pub button_rect_a: [Dword; 4],
    /// +0x80 (0x5C299C): `ViewerAndKinds.viewer_panel.button_rect_b[0]`: second button rectangle (`race::viewer_frame`).
    pub button_rect_b: [Dword; 4],
    /// +0x90 (0x5C29AC): `ViewerAndKinds.viewer_panel.button_rect_c[0]`: third button rectangle (`race::viewer_frame`).
    pub button_rect_c: [Dword; 4],
    /// +0xa0 (0x5C29BC): `ViewerAndKinds.viewer_panel.button_rect_d[0]`: fourth button rectangle (`race::viewer_frame`).
    pub button_rect_d: [Dword; 4],
    /// +0xb0 (0x5C29CC): `ViewerAndKinds.viewer_panel.bar_rect[0]`: the bar's rectangle (x, y, w at +8, h) (`race::viewer_frame`).
    pub bar_rect: [Dword; 4],
    /// +0xc0 (0x5C29DC): `ViewerAndKinds.viewer_panel.mark_rect[0]`: a mark's rectangle, x written each frame (`race::viewer_frame`).
    pub mark_rect: [Dword; 4],
    /// +0xd0 (0x5C29EC): `ViewerAndKinds.viewer_panel.place_rect[0]`: the place marker's rectangle, x written each frame (`race::viewer_frame`).
    pub place_rect: [Dword; 4],
    /// +0xe0 (0x5C29FC): `ViewerAndKinds.viewer_panel.item_place[0]`: the item quad's place, zero in the image (`race::draw_viewer_item`).
    pub item_place: [Dword; 4],
    /// +0xf0 (0x5C2A0C): `ViewerAndKinds.viewer_panel.screen_rect[0]`: the quad's screen rectangle, scratch, zero in the image (`race::draw_viewer_panel`, `drive::tyre_screen_parts`).
    pub screen_rect: [Dword; 4],
    /// +0x100 (0x5C2A1C): `ViewerAndKinds.viewer_panel.texture_rect[0]`: the quad's texture rectangle, scratch, zero in the image (`race::draw_viewer_panel`, `drive::tyre_screen_parts`, `race::draw_tyre_change`).
    pub texture_rect: [Dword; 4],
    /// +0x110 (0x5C2A2C): texture rectangles the viewer list's items point to (relocated pointers at 0x5C2580 ..); read by no code by name.
    pub viewer_item_rects_a: [Dword; 40],
    /// +0x1b0 (0x5C2ACC): `ViewerAndKinds.viewer_panel.mark_picture_rect[0]`: texture rectangle (249,0,7,29) of a mark (`race::viewer_frame`).
    pub mark_picture_rect: [Dword; 4],
    /// +0x1c0 (0x5C2ADC): a rectangle the viewer list points to (pointer at 0x5C25D0); read by no code by name.
    pub viewer_item_rect_b: [Dword; 4],
    /// +0x1d0 (0x5C2AEC): `ViewerAndKinds.viewer_panel.place_picture_rect[0]`: texture rectangle (48,48,48,48) of the place marker (`race::viewer_frame`).
    pub place_picture_rect: [Dword; 4],
    /// +0x1e0 (0x5C2AFC): texture rectangles the viewer list's items point to (pointers at 0x5C25F0 ..); read by no code by name.
    pub viewer_item_rects_c: [Dword; 76],
    /// +0x310 (0x5C2C2C): `ViewerAndKinds.viewer_panel.button_picture_rect[0]`: texture rectangle (0,240,16,16) of a button end (`race::viewer_frame`).
    pub button_picture_rect: [Dword; 4],
    /// +0x320 (0x5C2C3C): `ViewerAndKinds.viewer_panel.button_body_rect[0]`: texture rectangle (15,240,241,16) of a button body (`race::viewer_frame`).
    pub button_body_rect: [Dword; 4],
    /// +0x330 (0x5C2C4C): `ViewerAndKinds.viewer_panel.frame_picture_rect[0]` (20 bytes in IDA): texture rectangle (1,241,14,14) of the bar's frame, plus 0.5f as its fifth dword (`race::viewer_frame`).
    pub frame_picture_rect: [Dword; 5],
    /// +0x344 (0x5C2C60): `ViewerAndKinds.viewer_panel.one`: 1.0 (`race::tyre_change_gauge`).
    pub one: Float,
    /// +0x348 (0x5C2C64): `ViewerAndKinds.viewer_panel.texel_scale`: 1/256 (`drive::tyre_quad`, `drive::tyre_turned_quad`).
    pub texel_scale: Float,
    /// +0x34c (0x5C2C68): `ViewerAndKinds.viewer_panel.reciprocal_width_800`: 0.00125 = 1/800, what the screen's width is multiplied by to scale the tyre quad's x (`drive::tyre_quad`, `drive::tyre_turned_quad`).
    pub reciprocal_width_800: Float,
    /// +0x350 (0x5C2C6C): `ViewerAndKinds.viewer_panel.reciprocal_height_600`: 0.00167 = 1/600, what the screen's height is multiplied by to scale the tyre quad's y (`drive::tyre_quad`, `drive::tyre_turned_quad`).
    pub reciprocal_height_600: Float,
    /// +0x354 (0x5C2C70): `ViewerAndKinds.viewer_panel.switch_pictures_a[0]`: two relocated pointers to texture rectangles, a camera byte 2 or 3 picks one (`race::draw_viewer_item`).
    pub switch_pictures_a: [Dword; 2],
    /// +0x35c (0x5C2C78): `ViewerAndKinds.viewer_panel.switch_pictures_b[0]`: four relocated pointers to texture rectangles, by camera byte 5 (`race::draw_viewer_item`).
    pub switch_pictures_b: [Dword; 4],
    /// +0x36c (0x5C2C88): `ViewerAndKinds.viewer_panel.switch_pictures_c[0]`: three relocated pointers to texture rectangles, by camera byte 4 (`race::draw_viewer_item`).
    pub switch_pictures_c: [Dword; 3],
    /// +0x378 (0x5C2C94): `ViewerAndKinds.viewer_panel.patches_in_force`: the replay's physics patches are in force (`race::replay_patches`).
    pub patches_in_force: Dword,
    /// +0x37c (0x5C2C98): `ViewerAndKinds.viewer_panel.kept_patch_switches[0]`: the game's 13 patch switches kept while a replay's are in force (`race::replay_patches`).
    pub kept_patch_switches: [Dword; 13],
    /// +0x3b0 (0x5C2CCC): `g_replayRecording`: the frames are saved as pictures (`race::viewer`, `race::replay_screenshot`).
    pub replay_recording: Dword,
    /// +0x3b4 (0x5C2CD0): `ViewerAndKinds.viewer_panel.picture_clock`: the picture clock (ms), also read by `system::tick_count` (`race::replay_screenshot`).
    pub picture_clock: Dword,
    /// +0x3b8 (0x5C2CD4): `ViewerAndKinds.viewer_panel.pictures_started`: pictures started; `system::tick_count` overrides its clock when 1 (`race::replay_screenshot`).
    pub pictures_started: Dword,
    /// +0x3bc (0x5C2CD8): `ViewerAndKinds.viewer_panel.picture_folder`: the text rpl and a NUL, the pictures' folder (`race::start_frame_pictures`).
    pub picture_folder: Dword,
    /// +0x3c0 (0x5C2CDC): `aRpl000000Pcx`: the 15-byte name rpl\000000.pcx, taken in place; the last 3 bytes (0x5C2CE8 .. 0x5C2CEB) are not a whole dword (`race::replay_screenshot`).
    pub picture_name: [Dword; 3],
    /// +0x3cc (0x5C2CE8): the untyped tail: the viewer's texts and unaligned values, reached through `text` and the unaligned readers below.
    pub untyped_tail: [Dword; 167],
}

/// The piece: a field of `VIEWER_AND_KINDS` since 2026-10-29 (it left the race
/// block with its group, phase 13).
pub struct InGroup;

impl core::ops::Deref for InGroup {
    type Target = ViewerPanelBlock;
    fn deref(&self) -> &ViewerPanelBlock {
        &race().replay.viewer_and_kinds.viewer_panel
    }
}

pub static VIEWER_PANEL: InGroup = InGroup;

/// IDA 0x5C2CE8, the first byte of `untyped_tail`: the untyped tail with the viewer's texts and
/// its unaligned values.
const TAIL: usize = 0x5C2CE8;

/// The texts in `untyped_tail`, by IDA's address (for `ViewerPanelBlock::text`).
pub mod texts {
    /// "SVM: Release all textures"
    pub const SVM_RELEASE_TEXTURES: usize = 0x5C2DDF;
    /// "NET: Starting link"
    pub const NET_STARTING_LINK: usize = 0x5C2E04;
    /// "NET: Stopping link"
    pub const NET_STOPPING_LINK: usize = 0x5C2E17;
    /// "NET: Lets rally"
    pub const NET_LETS_RALLY: usize = 0x5C2E2A;
    /// "NET: Create dplay"
    pub const NET_CREATE_DPLAY: usize = 0x5C2E3A;
    /// "NET: Destroy dplay"
    pub const NET_DESTROY_DPLAY: usize = 0x5C2E4C;
    /// "NET: Find connections"
    pub const NET_FIND_CONNECTIONS: usize = 0x5C2E5F;
    /// "NET: Found"
    pub const NET_FOUND: usize = 0x5C2E75;
    /// "NET: Init connection index"
    pub const NET_INIT_CONNECTION_INDEX: usize = 0x5C2E80;
    /// "NET: Init"
    pub const NET_INIT: usize = 0x5C2E9B;
    /// "NET: Destroy connection"
    pub const NET_DESTROY_CONNECTION: usize = 0x5C2EA5;
    /// "NET: DPCA"
    pub const NET_DPCA: usize = 0x5C2EBD;
    /// "NET: Find sessions"
    pub const NET_FIND_SESSIONS: usize = 0x5C2EC7;
    /// "NET: Host"
    pub const NET_HOST: usize = 0x5C2EDA;
    /// "NET: Open"
    pub const NET_OPEN: usize = 0x5C2EE4;
    /// "NET: Host complete"
    pub const NET_HOST_COMPLETE: usize = 0x5C2EEE;
    /// "NET: Join session"
    pub const NET_JOIN_SESSION: usize = 0x5C2F01;
    /// "NET: Join complete"
    pub const NET_JOIN_COMPLETE: usize = 0x5C2F13;
    /// "NET: Disable new players"
    pub const NET_DISABLE_NEW_PLAYERS: usize = 0x5C2F26;
    /// "NET: Create dplobby"
    pub const NET_CREATE_DLOBBY: usize = 0x5C2F5A;
    /// "NET: Create player"
    pub const NET_CREATE_PLAYER: usize = 0x5C2F6E;
}

impl ViewerPanelBlock {
    /// The text of `texts` that starts at IDA's `ida` in the untyped tail, up to its NUL (or the
    /// tail's end).
    pub fn text(&self, ida: usize) -> Vec<u8> {
        assert!((TAIL..TAIL + 4 * 167).contains(&ida));
        let at = ida - TAIL;
        (at..4 * 167)
            .map(|k| (self.untyped_tail[k / 4].get() >> (8 * (k % 4))) as u8)
            .take_while(|&byte| byte != 0)
            .collect()
    }

    /// Where the byte at IDA `ida` is, inside the untyped tail `untyped_tail` (the tests).
    #[cfg(test)]
    pub fn text_ptr(&self, ida: usize) -> crate::pointers::GamePtr {
        assert!((TAIL..TAIL + 4 * 167).contains(&ida));
        crate::pointers::GamePtr::to(&self.untyped_tail).offset((ida - TAIL) as u32)
    }

    /// `ViewerAndKinds.viewer_panel.untyped_tail[0]` (unaligned, in the tail): 1 when the developer's keys work in the viewer.
    pub fn developer_keys(&self) -> u32 {
        self.tail_read(0x5C2CEB)
    }

    /// Sets `ViewerAndKinds.viewer_panel.untyped_tail[0]` (the `wrreplayenable` setting writes it).
    pub fn set_developer_keys(&self, value: u32) {
        self.tail_write(0x5C2CEB, value);
    }

    /// `ViewerAndKinds.viewer_panel.untyped_tail[1]` (unaligned): pictures a second, 1 .. 100.
    pub fn picture_rate(&self) -> u32 {
        self.tail_read(0x5C2CEF)
    }

    pub fn set_picture_rate(&self, value: u32) {
        self.tail_write(0x5C2CEF, value);
    }

    /// `ViewerAndKinds.viewer_panel.untyped_tail[2]` (unaligned): ms a picture.
    pub fn picture_time(&self) -> u32 {
        self.tail_read(0x5C2CF3)
    }

    pub fn set_picture_time(&self, value: u32) {
        self.tail_write(0x5C2CF3, value);
    }

    /// `ViewerAndKinds.viewer_panel.camera_name_ids[0]` ..: the text id of camera `camera` (47 of them; the original does not
    /// check the number).
    pub fn camera_name(&self, camera: usize) -> u16 {
        // The names are the block's first bytes; a number past them reads on in the block.
        crate::pointers::load_value::<u16>(self, 2 * camera)
            .unwrap_or_else(|e| crate::fatal::raise::end("a camera's name", e))
    }

    /// The rectangle of the picture `n` in switch table `table` (0 `ViewerAndKinds.viewer_panel.switch_pictures_a[0]`, 1
    /// `ViewerAndKinds.viewer_panel.switch_pictures_b[0]`, 2 `ViewerAndKinds.viewer_panel.switch_pictures_c[0]`) is (the tables keep IDA's addresses as names). The three tables
    /// are one run of nine names, which an `n` past a table's end reads on into, as the original; past the run,
    /// or a name off the panel, is refused.
    pub fn switch_picture(
        &self,
        table: usize,
        n: u32,
    ) -> Result<[u32; 4], crate::errors::BadInput> {
        let run: [&Dword; 9] = [
            &self.switch_pictures_a[0],
            &self.switch_pictures_a[1],
            &self.switch_pictures_b[0],
            &self.switch_pictures_b[1],
            &self.switch_pictures_b[2],
            &self.switch_pictures_b[3],
            &self.switch_pictures_c[0],
            &self.switch_pictures_c[1],
            &self.switch_pictures_c[2],
        ];
        let first = [0usize, 2, 6]
            .get(table)
            .ok_or(crate::errors::BadInput("a switch table past the 3"))?;
        let cell = usize::try_from(n)
            .ok()
            .and_then(|n| run.get(first + n))
            .ok_or(crate::errors::BadInput(
                "a camera byte past the switch pictures",
            ))?;
        self.rectangle(cell.get() as usize)
    }

    /// The rectangle (four dwords) the IDA address `ida` in the panel names; an address off the
    /// panel is refused.
    pub fn rectangle(&self, ida: usize) -> Result<[u32; 4], crate::errors::BadInput> {
        if !(START..END).contains(&ida) {
            return Err(crate::errors::BadInput(
                "a switch picture's name off the panel",
            ));
        }
        crate::pointers::load_value(self, ida - START)
    }

    /// The dword at IDA's `ida` in the tail: unaligned, so made of the two cells it spans.
    fn tail_read(&self, ida: usize) -> u32 {
        let (n, shift) = Self::tail_cells(ida);
        let low = self.untyped_tail[n].get();
        match shift {
            0 => low,
            _ => (low >> shift) | (self.untyped_tail[n + 1].get() << (32 - shift)),
        }
    }

    /// `tail_read`'s write: the bytes of the two cells it spans that the dword covers.
    fn tail_write(&self, ida: usize, value: u32) {
        let (n, shift) = Self::tail_cells(ida);
        if shift == 0 {
            self.untyped_tail[n].set(value);
            return;
        }
        let (low, high) = (&self.untyped_tail[n], &self.untyped_tail[n + 1]);
        low.set(low.get() & !(u32::MAX << shift) | value << shift);
        high.set(high.get() & (u32::MAX << shift) | value >> (32 - shift));
    }

    /// The tail's cell holding IDA's `ida` and the dword's bit shift in it.
    fn tail_cells(ida: usize) -> (usize, u32) {
        assert!((TAIL..TAIL + 4 * 166).contains(&ida));
        ((ida - TAIL) / 4, 8 * ((ida - TAIL) % 4) as u32)
    }
}

/// IDA 0x5C291C, the first byte of the piece.
pub const START: usize = 0x5C291C;
/// IDA 0x5C2F84, the end of the piece (the marks' settings follow).
pub const END: usize = 0x5C2F84;

const _: () = assert!(END - START == core::mem::size_of::<ViewerPanelBlock>());

/// Where IDA 0x5C291C is now (its group's static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(
        &race().replay.viewer_and_kinds.viewer_panel,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn tail_dwords_read_and_write_unaligned() {
        let _g = crate::store::test_lock();
        let v = &*VIEWER_PANEL;
        let kept: Vec<u32> = v
            .untyped_tail
            .iter()
            .map(crate::session::Dword::get)
            .collect();
        for (k, ida) in [TAIL, TAIL + 3, TAIL + 7, TAIL + 11]
            .into_iter()
            .enumerate()
        {
            for (n, c) in v.untyped_tail.iter().enumerate() {
                c.set(0xA5A5_0000 | n as u32);
            }
            let before: Vec<u8> = v.text_ptr(TAIL).read_bytes(24, "the tail").unwrap();
            let value = 0x1122_3344 + k as u32;
            v.tail_write(ida, value);
            let after: Vec<u8> = v.text_ptr(TAIL).read_bytes(24, "the tail").unwrap();
            let mut want = before.clone();
            want[ida - TAIL..ida - TAIL + 4].copy_from_slice(&value.to_le_bytes());
            assert_eq!(after, want, "{ida:#x}");
            assert_eq!(v.tail_read(ida), value);
        }
        for (c, k) in v.untyped_tail.iter().zip(kept) {
            c.set(k);
        }
    }

    #[test]
    fn the_switch_tables_name_their_pictures_as_the_image_does() {
        let v = &*VIEWER_PANEL;
        let name = |k: usize| {
            crate::test_support::first_values::dword(
                "race.owner.RACE.replay.viewer_and_kinds.viewer_panel.switch_pictures_a[0]/36",
                k,
            )
        };
        for k in 0..9usize {
            // The three tables hold two, four and three names, one run.
            let (table, n) = match k {
                0..2 => (0, k),
                2..6 => (1, k - 2),
                _ => (2, k - 6),
            };
            let cell = match table {
                0 => &v.switch_pictures_a[n],
                1 => &v.switch_pictures_b[n],
                _ => &v.switch_pictures_c[n],
            };
            assert_eq!(cell.get(), name(k));
            assert_eq!(
                v.switch_picture(table, n as u32).unwrap(),
                v.rectangle(name(k) as usize).unwrap()
            );
        }
        assert!(v.switch_picture(0, 9).is_err());
        assert!(v.switch_picture(3, 0).is_err());
    }

    #[test]
    fn fields_are_at_the_games_offsets() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(offset_of!(ViewerPanelBlock, bar_frame_rect), 0x60);
        assert_eq!(offset_of!(ViewerPanelBlock, mark_rect), 0xc0);
        assert_eq!(offset_of!(ViewerPanelBlock, place_picture_rect), 0x1d0);
        assert_eq!(offset_of!(ViewerPanelBlock, reciprocal_width_800), 0x34c);
        assert_eq!(offset_of!(ViewerPanelBlock, kept_patch_switches), 0x37c);
    }
}
