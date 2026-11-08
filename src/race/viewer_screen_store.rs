//! The replay viewer's screen (IDA 0x5C291C .. 0x5C2FE4; phase 7, typed): the cameras' text ids, the bar, buttons, marks and place rectangles (scratch ones written each frame), the quad scale floats, the picture switches, the replay patch switches kept, the frame-picture recording state (three dwords at odd addresses), the NET and display text strings between 0x5C2CF7 and 0x5C2F81 (left as unknown bytes), the view marks' state and the listener place and turn. Names are readings unless IDA's.

use crate::race::owner::race;
use crate::session::{Dword, Float};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ViewerPanelBlock {
    /// +0x0 (0x5C291C): not named.
    pub unknown_0: [Dword; 24],
    /// +0x60 (0x5C297C): `dword_5C297C`: rectangle (0,0,800,32) of the viewer bar's frame (`race::viewer_frame`).
    pub bar_frame_rect: [Dword; 4],
    /// +0x70 (0x5C298C): `dword_5C298C`: first button rectangle (`race::viewer_frame`).
    pub button_rect_a: [Dword; 4],
    /// +0x80 (0x5C299C): `dword_5C299C`: second button rectangle (`race::viewer_frame`).
    pub button_rect_b: [Dword; 4],
    /// +0x90 (0x5C29AC): `dword_5C29AC`: third button rectangle (`race::viewer_frame`).
    pub button_rect_c: [Dword; 4],
    /// +0xa0 (0x5C29BC): `dword_5C29BC`: fourth button rectangle (`race::viewer_frame`).
    pub button_rect_d: [Dword; 4],
    /// +0xb0 (0x5C29CC): `dword_5C29CC`: the bar's rectangle (x, y, w at +8, h) (`race::viewer_frame`).
    pub bar_rect: [Dword; 4],
    /// +0xc0 (0x5C29DC): `dword_5C29DC`: a mark's rectangle, x written each frame (`race::viewer_frame`).
    pub mark_rect: [Dword; 4],
    /// +0xd0 (0x5C29EC): `dword_5C29EC`: the place marker's rectangle, x written each frame (`race::viewer_frame`).
    pub place_rect: [Dword; 4],
    /// +0xe0 (0x5C29FC): `dword_5C29FC`: the item quad's place, zero in the image (`race::draw_viewer_item`).
    pub item_place: [Dword; 4],
    /// +0xf0 (0x5C2A0C): `dword_5C2A0C`: the quad's screen rectangle, scratch, zero in the image (`race::draw_viewer_panel`, `drive::tyre_screen_parts`).
    pub screen_rect: [Dword; 4],
    /// +0x100 (0x5C2A1C): `dword_5C2A1C`: the quad's texture rectangle, scratch, zero in the image (`race::draw_viewer_panel`, `drive::tyre_screen_parts`, `race::draw_tyre_change`).
    pub texture_rect: [Dword; 4],
    /// +0x110 (0x5C2A2C): not named.
    pub unknown_110: [Dword; 40],
    /// +0x1b0 (0x5C2ACC): `dword_5C2ACC`: texture rectangle (249,0,7,29) of a mark (`race::viewer_frame`).
    pub mark_picture_rect: [Dword; 4],
    /// +0x1c0 (0x5C2ADC): not named.
    pub unknown_1c0: [Dword; 4],
    /// +0x1d0 (0x5C2AEC): `dword_5C2AEC`: texture rectangle (48,48,48,48) of the place marker (`race::viewer_frame`).
    pub place_picture_rect: [Dword; 4],
    /// +0x1e0 (0x5C2AFC): not named.
    pub unknown_1e0: [Dword; 76],
    /// +0x310 (0x5C2C2C): `dword_5C2C2C`: texture rectangle (0,240,16,16) of a button end (`race::viewer_frame`).
    pub button_picture_rect: [Dword; 4],
    /// +0x320 (0x5C2C3C): `dword_5C2C3C`: texture rectangle (15,240,241,16) of a button body (`race::viewer_frame`).
    pub button_body_rect: [Dword; 4],
    /// +0x330 (0x5C2C4C): `dword_5C2C4C` (20 bytes in IDA): texture rectangle (1,241,14,14) of the bar's frame, plus 0.5f as its fifth dword (`race::viewer_frame`).
    pub frame_picture_rect: [Dword; 5],
    /// +0x344 (0x5C2C60): `flt_5C2C60`: 1.0 (`race::tyre_change_gauge`).
    pub one: Float,
    /// +0x348 (0x5C2C64): `flt_5C2C64`: 1/256 (`drive::tyre_quad`, `drive::tyre_turned_quad`).
    pub texel_scale: Float,
    /// +0x34c (0x5C2C68): `flt_5C2C68`: 0.00125 (a reading) (`drive::tyre_quad`, `drive::tyre_turned_quad`).
    pub texel_scale_small: Float,
    /// +0x350 (0x5C2C6C): `flt_5C2C6C`: 0.00167 (a reading) (`drive::tyre_quad`, `drive::tyre_turned_quad`).
    pub texel_scale_smaller: Float,
    /// +0x354 (0x5C2C70): `off_5C2C70`: two relocated pointers to texture rectangles, a camera byte 2 or 3 picks one (`race::draw_viewer_item`).
    pub switch_pictures_a: [Dword; 2],
    /// +0x35c (0x5C2C78): `off_5C2C78`: four relocated pointers to texture rectangles, by camera byte 5 (`race::draw_viewer_item`).
    pub switch_pictures_b: [Dword; 4],
    /// +0x36c (0x5C2C88): `off_5C2C88`: three relocated pointers to texture rectangles, by camera byte 4 (`race::draw_viewer_item`).
    pub switch_pictures_c: [Dword; 3],
    /// +0x378 (0x5C2C94): `dword_5C2C94`: the replay's physics patches are in force (`race::replay_patches`).
    pub patches_in_force: Dword,
    /// +0x37c (0x5C2C98): `unk_5C2C98`: the game's 13 patch switches kept while a replay's are in force (`race::replay_patches`).
    pub kept_patch_switches: [Dword; 13],
    /// +0x3b0 (0x5C2CCC): `g_replayRecording`: the frames are saved as pictures (`race::viewer`, `race::replay_screenshot`).
    pub replay_recording: Dword,
    /// +0x3b4 (0x5C2CD0): `dword_5C2CD0`: the picture clock (ms), also read by `system::tick_count` (`race::replay_screenshot`).
    pub picture_clock: Dword,
    /// +0x3b8 (0x5C2CD4): `dword_5C2CD4`: pictures started; `system::tick_count` overrides its clock when 1 (`race::replay_screenshot`).
    pub pictures_started: Dword,
    /// +0x3bc (0x5C2CD8): `off_5C2CD8`: the text rpl and a NUL, the pictures' folder (`race::start_frame_pictures`).
    pub picture_folder: Dword,
    /// +0x3c0 (0x5C2CDC): `aRpl000000Pcx`: the 15-byte name rpl\000000.pcx, taken in place; the last 3 bytes (0x5C2CE8 .. 0x5C2CEB) are not a whole dword (`race::replay_screenshot`).
    pub picture_name: [Dword; 3],
    /// +0x3cc (0x5C2CE8): not named.
    pub unknown_3cc: [Dword; 167],
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

/// IDA 0x5C2CE8, the first byte of `unknown_3cc`: the untyped tail with the viewer's texts and
/// its unaligned values.
const TAIL: usize = 0x5C2CE8;

/// The texts in `unknown_3cc`, by IDA's address (for `ViewerPanelBlock::text_ptr`).
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
    /// Where the byte at IDA `ida` is, inside the untyped tail `unknown_3cc` (the texts of
    /// `texts`, the unaligned values below).
    pub fn text_ptr(&self, ida: usize) -> crate::pointers::GamePtr {
        assert!((TAIL..TAIL + 4 * 167).contains(&ida));
        crate::pointers::GamePtr::to(&self.unknown_3cc).offset((ida - TAIL) as u32)
    }

    /// `dword_5C2CEB` (unaligned, in the tail): 1 when the developer's keys work in the viewer.
    pub fn developer_keys(&self) -> u32 {
        self.tail_read(0x5C2CEB)
    }

    /// Sets `dword_5C2CEB` (the `wrreplayenable` setting writes it).
    pub fn set_developer_keys(&self, value: u32) {
        self.tail_write(0x5C2CEB, value);
    }

    /// `dword_5C2CEF` (unaligned): pictures a second, 1 .. 100.
    pub fn picture_rate(&self) -> u32 {
        self.tail_read(0x5C2CEF)
    }

    pub fn set_picture_rate(&self, value: u32) {
        self.tail_write(0x5C2CEF, value);
    }

    /// `dword_5C2CF3` (unaligned): ms a picture.
    pub fn picture_time(&self) -> u32 {
        self.tail_read(0x5C2CF3)
    }

    pub fn set_picture_time(&self, value: u32) {
        self.tail_write(0x5C2CF3, value);
    }

    /// `word_5C291C` ..: the text id of camera `camera` (47 of them; the original does not
    /// check the number).
    pub fn camera_name(&self, camera: usize) -> u16 {
        // The names are the block's first bytes; a number past them reads on in the block.
        crate::pointers::GamePtr::to(self)
            .offset((2 * camera) as u32)
            .read("a camera's name")
            .unwrap_or_else(|e| crate::fatal::raise::end("a camera's name", e))
    }

    /// Where the picture `n` places on in switch table `table` (0 `off_5C2C70`, 1
    /// `off_5C2C78`, 2 `off_5C2C88`) is (phase 13: the tables keep IDA's addresses as names,
    /// and `moved::point_into_homes` passes them over). The three tables are one run of nine
    /// names, which an `n` past a table's end reads on into, as the original; past the run,
    /// or a name off the panel, is refused.
    pub fn switch_picture(
        &self,
        table: usize,
        n: u32,
    ) -> Result<crate::pointers::GamePtr, crate::errors::BadInput> {
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
        let ida = cell.get() as usize;
        if !(START..END).contains(&ida) {
            return Err(crate::errors::BadInput(
                "a switch picture's name off the panel",
            ));
        }
        crate::moved::named(ida)
    }

    /// The dword at IDA's `ida` in the tail: unaligned, so made of the two cells it spans.
    fn tail_read(&self, ida: usize) -> u32 {
        let (n, shift) = Self::tail_cells(ida);
        let low = self.unknown_3cc[n].get();
        match shift {
            0 => low,
            _ => (low >> shift) | (self.unknown_3cc[n + 1].get() << (32 - shift)),
        }
    }

    /// `tail_read`'s write: the bytes of the two cells it spans that the dword covers.
    fn tail_write(&self, ida: usize, value: u32) {
        let (n, shift) = Self::tail_cells(ida);
        if shift == 0 {
            self.unknown_3cc[n].set(value);
            return;
        }
        let (low, high) = (&self.unknown_3cc[n], &self.unknown_3cc[n + 1]);
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
            .unknown_3cc
            .iter()
            .map(crate::session::Dword::get)
            .collect();
        for (k, ida) in [0x5C2CE8, 0x5C2CEB, 0x5C2CEF, 0x5C2CF3]
            .into_iter()
            .enumerate()
        {
            for (n, c) in v.unknown_3cc.iter().enumerate() {
                c.set(0xA5A5_0000 | n as u32);
            }
            let before: Vec<u8> = v.text_ptr(0x5C2CE8).read_bytes(24, "the tail").unwrap();
            let value = 0x1122_3344 + k as u32;
            v.tail_write(ida, value);
            let after: Vec<u8> = v.text_ptr(0x5C2CE8).read_bytes(24, "the tail").unwrap();
            let mut want = before.clone();
            want[ida - TAIL..ida - TAIL + 4].copy_from_slice(&value.to_le_bytes());
            assert_eq!(after, want, "{ida:#x}");
            assert_eq!(v.tail_read(ida), value);
        }
        for (c, k) in v.unknown_3cc.iter().zip(kept) {
            c.set(k);
        }
    }

    #[test]
    fn the_switch_tables_name_their_pictures_as_the_image_does() {
        let v = &*VIEWER_PANEL;
        let name =
            |ida: usize| u32::from_le_bytes(crate::moved::image_bytes(ida, 4).try_into().unwrap());
        for k in 0..9usize {
            let ida = 0x5C2C70 + 4 * k;
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
            assert_eq!(cell.get(), name(ida));
            assert_eq!(
                v.switch_picture(table, n as u32).unwrap().addr() as usize,
                crate::moved::place(name(ida) as usize)
            );
        }
        assert!(v.switch_picture(0, 9).is_err());
        assert!(v.switch_picture(3, 0).is_err());
    }

    #[test]
    fn piece_sits_on_the_race_block() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(
            offset_of!(ViewerPanelBlock, bar_frame_rect),
            0x5C297C - START
        );
        assert_eq!(offset_of!(ViewerPanelBlock, mark_rect), 0x5C29DC - START);
        assert_eq!(
            offset_of!(ViewerPanelBlock, place_picture_rect),
            0x5C2AEC - START
        );
        assert_eq!(
            offset_of!(ViewerPanelBlock, texel_scale_small),
            0x5C2C68 - START
        );
        assert_eq!(
            offset_of!(ViewerPanelBlock, kept_patch_switches),
            0x5C2C98 - START
        );
    }
}
