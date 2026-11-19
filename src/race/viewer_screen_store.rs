//! The replay viewer's screen: the cameras' text ids, the bar, buttons, marks and place rectangles
//! (scratch ones written each frame), the quad scale floats, the picture switches, the replay patch
//! switches kept, the frame-picture recording state (three unaligned dwords), the NET and display
//! text strings (left as unknown bytes), the view marks' state and the listener place and turn.

use crate::memory::cells::ZeroCells;
use crate::memory::session::{PlainDword, PlainFloat};
use crate::race::first_cells::{plain_dwords, plain_dwords_of_text};
#[cfg(test)]
use crate::test_support::own::race;

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells, rc2k_derive::ByteView)]
pub struct ViewerPanelBlock {
    /// +0x0: 16-bit text ids of the camera names, read and written by `race::draw_viewer_panel`
    /// and the camera actions; reached through `camera_name`, not by this field.
    pub camera_name_ids: [PlainDword; 24],
    /// +0x60: rectangle (0,0,800,32) of the viewer
    /// bar's frame (`race::viewer_frame`).
    pub bar_frame_rect: [PlainDword; 4],
    /// +0x70: first button rectangle
    /// (`race::viewer_frame`).
    pub button_rect_a: [PlainDword; 4],
    /// +0x80: second button rectangle
    /// (`race::viewer_frame`).
    pub button_rect_b: [PlainDword; 4],
    /// +0x90: third button rectangle
    /// (`race::viewer_frame`).
    pub button_rect_c: [PlainDword; 4],
    /// +0xa0: fourth button rectangle
    /// (`race::viewer_frame`).
    pub button_rect_d: [PlainDword; 4],
    /// +0xb0: the bar's rectangle (x, y, w at +8, h)
    /// (`race::viewer_frame`).
    pub bar_rect: [PlainDword; 4],
    /// +0xc0: a mark's rectangle, x written each frame
    /// (`race::viewer_frame`).
    pub mark_rect: [PlainDword; 4],
    /// +0xd0: the place marker's rectangle, x written
    /// each frame (`race::viewer_frame`).
    pub place_rect: [PlainDword; 4],
    /// +0xe0: the item quad's place, zero at the start
    /// (`race::draw_viewer_item`).
    pub item_place: [PlainDword; 4],
    /// +0xf0: the quad's screen rectangle, scratch,
    /// zero at the start (`race::draw_viewer_panel`, `drive::tyre_screen_parts`).
    pub screen_rect: [PlainDword; 4],
    /// +0x100: the quad's texture rectangle,
    /// scratch, zero at the start (`race::draw_viewer_panel`, `drive::tyre_screen_parts`,
    /// `race::draw_tyre_change`).
    pub texture_rect: [PlainDword; 4],
    /// +0x110: texture rectangles the viewer list's items point to; never read by name.
    pub viewer_item_rects_a: [PlainDword; 40],
    /// +0x1b0: texture rectangle (249,0,7,29)
    /// of a mark (`race::viewer_frame`).
    pub mark_picture_rect: [PlainDword; 4],
    /// +0x1c0: a rectangle the viewer list points to; never read by name.
    pub viewer_item_rect_b: [PlainDword; 4],
    /// +0x1d0: texture rectangle (48,48,48,48)
    /// of the place marker (`race::viewer_frame`).
    pub place_picture_rect: [PlainDword; 4],
    /// +0x1e0: texture rectangles the viewer list's items point to; never read by name.
    pub viewer_item_rects_c: [PlainDword; 76],
    /// +0x310: texture rectangle
    /// (0,240,16,16) of a button end (`race::viewer_frame`).
    pub button_picture_rect: [PlainDword; 4],
    /// +0x320: texture rectangle (15,240,241,16)
    /// of a button body (`race::viewer_frame`).
    pub button_body_rect: [PlainDword; 4],
    /// +0x330: the bar frame's texture rectangle (1,241,14,14), 20 bytes: plus 0.5f as its fifth
    /// dword (`race::viewer_frame`).
    pub frame_picture_rect: [PlainDword; 5],
    /// +0x344: the constant 1.0 (`race::tyre_change_gauge`).
    pub one: PlainFloat,
    /// +0x348: the texel scale, 1/256 (`drive::tyre_quad`, `drive::tyre_turned_quad`).
    pub texel_scale: PlainFloat,
    /// +0x34c: 0.00125 = 1/800, what the screen's width is multiplied by to scale the tyre quad's
    /// x (`drive::tyre_quad`, `drive::tyre_turned_quad`).
    pub reciprocal_width_800: PlainFloat,
    /// +0x350: 0.00167 = 1/600, what the screen's height is multiplied by to scale the tyre
    /// quad's y (`drive::tyre_quad`, `drive::tyre_turned_quad`).
    pub reciprocal_height_600: PlainFloat,
    /// +0x354: two names of texture rectangles; a camera byte 2 or 3 picks one
    /// (`race::draw_viewer_item`).
    pub switch_pictures_a: [PlainDword; 2],
    /// +0x35c: four names of texture rectangles, by camera byte 5 (`race::draw_viewer_item`).
    pub switch_pictures_b: [PlainDword; 4],
    /// +0x36c: three names of texture rectangles, by camera byte 4 (`race::draw_viewer_item`).
    pub switch_pictures_c: [PlainDword; 3],
    /// +0x378: whether the replay's physics patches are in force (`race::replay_patches`).
    pub patches_in_force: PlainDword,
    /// +0x37c: the 13 patch switches kept while a replay's are in force (`race::replay_patches`).
    pub kept_patch_switches: [PlainDword; 13],
    /// +0x3b0: the frames are saved as pictures (`race::viewer`, `race::replay_screenshot`).
    pub replay_recording: PlainDword,
    /// +0x3b4: the picture clock, moved to `clock::tick_count::PICTURE_CLOCK.clock`; the bytes
    /// stay, unused.
    pub picture_clock_unused: PlainDword,
    /// +0x3b8: whether the pictures have started, moved to
    /// `clock::tick_count::PICTURE_CLOCK.started`; the bytes stay, unused.
    pub pictures_started_unused: PlainDword,
    /// +0x3bc: the pictures' folder: the text rpl and a NUL (`race::start_frame_pictures`).
    pub picture_folder: PlainDword,
    /// +0x3c0: the 15-byte picture name rpl\000000.png, taken in place; the last 3 bytes
    /// are not a whole dword (`race::replay_screenshot`).
    pub picture_name: [PlainDword; 3],
    /// +0x3cc: the untyped tail: the viewer's texts and unaligned values, reached through `text`
    /// and the unaligned readers below.
    pub untyped_tail: [PlainDword; 167],
}

impl ViewerPanelBlock {
    /// The viewer panel at the start: its rectangles, camera names, picture numbers and the
    /// texts it keeps after them.
    pub const fn new() -> Self {
        ViewerPanelBlock {
            camera_name_ids: plain_dwords([
                0x1CA01D1, 0x1C901D0, 0x1B801B6, 0x1C201C1, 0x1AA01A9, 0x1D201AB, 0x1D401D3,
                0x1A601D5, 0x1A801A7, 0x1AD01AC, 0x1AF01AE, 0x1B101B0, 0x1B301B2, 0x1B501B4,
                0x1B901B7, 0x1BB01BA, 0x1BD01BC, 0x1BF01BE, 0x1C301C0, 0x1C501C4, 0x1C701C6,
                0x1CB01C8, 0x1CD01CC, 0x1CE,
            ]),
            bar_frame_rect: plain_dwords([0, 0, 0x320, 0x20]),
            button_rect_a: plain_dwords([0, 0x1F, 0x10, 0x10]),
            button_rect_b: plain_dwords([0xF, 0x1F, 0x182, 0x10]),
            button_rect_c: plain_dwords([0x190, 0x1F, 0x10, 0x10]),
            button_rect_d: plain_dwords([0x19F, 0x1F, 0x181, 0x10]),
            bar_rect: plain_dwords([0x6C, 3, 0x248, 0x1A]),
            mark_rect: plain_dwords([0, 2, 7, 0x1C]),
            place_rect: plain_dwords([0, 4, 0x18, 0x18]),
            viewer_item_rects_a: plain_dwords([
                0, 0, 0x30, 0x30, 0x30, 0, 0xFFFFFFD0, 0x30, 0x30, 0, 0x30, 0x30, 0x60, 0,
                0xFFFFFFD0, 0x30, 0x60, 0, 0x30, 0x30, 0x90, 0, 0xFFFFFFD0, 0x30, 0x91, 0, 0x2F,
                0x30, 0xBF, 0, 0xFFFFFFD1, 0x30, 0xC1, 0, 0x30, 0x30, 0xF0, 0, 0xFFFFFFD0, 0x30,
            ]),
            mark_picture_rect: plain_dwords([0xF9, 0, 7, 0x1D]),
            viewer_item_rect_b: plain_dwords([0, 0x30, 0x30, 0x30]),
            place_picture_rect: [const { PlainDword::new(0x30) }; 4],
            viewer_item_rects_c: plain_dwords([
                0x60, 0x30, 0x30, 0x30, 0x90, 0x30, 0x30, 0x30, 0xC0, 0x30, 0x14, 0x30, 0xD9, 0x30,
                0x22, 0x30, 0, 0x60, 0x30, 0x30, 0x30, 0x60, 0x30, 0x30, 0x60, 0x60, 0x30, 0x30,
                0x90, 0x60, 0x30, 0x30, 0xC0, 0x60, 0x30, 0x30, 0, 0x90, 0x30, 0x30, 0x30, 0x90,
                0x30, 0x30, 0x60, 0x90, 0x30, 0x30, 0x90, 0x90, 0x30, 0x30, 0xC0, 0x90, 0x30, 0x30,
                0, 0xC0, 0x30, 0x30, 0x30, 0xC0, 0x30, 0x30, 0x60, 0xC0, 0x30, 0x30, 0x90, 0xC0,
                0x30, 0x30, 0xC0, 0xC0, 0x30, 0x30,
            ]),
            button_picture_rect: plain_dwords([0, 0xF0, 0x10, 0x10]),
            button_body_rect: plain_dwords([0xF, 0xF0, 0xF1, 0x10]),
            frame_picture_rect: plain_dwords([1, 0xF1, 0xE, 0xE, 0x3F000000]),
            one: PlainFloat::new(1.0),
            texel_scale: PlainFloat::new(0.00390625),
            reciprocal_width_800: PlainFloat::new(0.00125),
            reciprocal_height_600: PlainFloat::new(0.0016666667),
            switch_pictures_a: plain_dwords([0x5C2C0C, 0x5C2B1C]),
            switch_pictures_b: plain_dwords([0x5C2B3C, 0x5C2B4C, 0x5C2B5C, 0x5C2B6C]),
            switch_pictures_c: plain_dwords([0x5C2B7C, 0x5C2B8C, 0x5C2B9C]),
            picture_folder: PlainDword::new(0x6C7072),
            picture_name: plain_dwords_of_text::<3>(b"rpl\\000000.p"),
            untyped_tail: plain_dwords_of_text::<167>(TAIL_TEXT),
            ..ZeroCells::ZERO
        }
    }
}

/// The panel as a field of the replay's `viewer_and_kinds`.
#[cfg(test)]
pub struct InGroup;

#[cfg(test)]
impl core::ops::Deref for InGroup {
    type Target = ViewerPanelBlock;
    fn deref(&self) -> &ViewerPanelBlock {
        &race().replay.viewer_and_kinds.viewer_panel
    }
}

#[cfg(test)]
pub static VIEWER_PANEL: InGroup = InGroup;

/// The tail's first bytes at the start: the last bytes of the picture name, the viewer's
/// settings (`wrreplayenable`, `wrreplayrate`, the picture time), and the texts the network code
/// logs (`texts`), zero-padded to the tail's length.
const TAIL_TEXT: &[u8] = b"ng\0\x01\0\0\0d\0\0\0\n\0\0\0Finding 3d hardware\0 dd:\0=\0:\x003d:\0primary card\0secondary card\0VID: Mode for game=\0VID: Mode for main=\0VID: Open ddraw\0VID: Close ddraw\0Getting available video modes\0SVM:\0SVM: OK\0SVM: FAIL\0SDM:\0 SDM return=\0SVM: Release video screens\0SVM: Release all textures\0OpenWindow\0NET: Starting link\0NET: Stopping link\0NET: Lets rally\0NET: Create dplay\0NET: Destroy dplay\0NET: Find connections\0NET: Found\0NET: Init connection index\0NET: Init\0NET: Destroy connection\0NET: DPCA\0NET: Find sessions\0NET: Host\0NET: Open\0NET: Host complete\0NET: Join session\0NET: Join complete\0NET: Disable new players\0NET: Lobby launch detected\0NET: Create dplobby\0NET: Create player";

/// The network code's log texts (`texts`): the viewer panel's tail holds them from the start and
/// nothing writes there (the viewer's three settings stand before them), so the code reads them
/// from the start values rather than from the panel. Plain data: the network's receiving thread
/// reads them beside the game thread.
pub struct NetTexts(&'static [u8]);

/// The network's log texts as they start.
pub static NET_TEXTS: NetTexts = NetTexts(TAIL_TEXT);

impl NetTexts {
    /// The text of `texts` that starts at byte `at` of the untyped tail, up to its NUL (or the
    /// tail's end).
    pub fn text(&self, at: usize) -> Vec<u8> {
        assert!(at < 4 * 167);
        self.0
            .get(at..)
            .unwrap_or_default()
            .iter()
            .copied()
            .take_while(|&byte| byte != 0)
            .collect()
    }
}

/// The texts in `untyped_tail`, by their first byte's place in it (for `NetTexts::text`).
pub mod texts {
    /// "NET: Starting link"
    pub const NET_STARTING_LINK: usize = 0x11C;
    /// "NET: Stopping link"
    pub const NET_STOPPING_LINK: usize = 0x12F;
    /// "NET: Lets rally"
    pub const NET_LETS_RALLY: usize = 0x142;
    /// "NET: Create dplay"
    pub const NET_CREATE_DPLAY: usize = 0x152;
    /// "NET: Destroy dplay"
    pub const NET_DESTROY_DPLAY: usize = 0x164;
    /// "NET: Find connections"
    pub const NET_FIND_CONNECTIONS: usize = 0x177;
    /// "NET: Found"
    pub const NET_FOUND: usize = 0x18D;
    /// "NET: Init connection index"
    pub const NET_INIT_CONNECTION_INDEX: usize = 0x198;
    /// "NET: Init"
    pub const NET_INIT: usize = 0x1B3;
    /// "NET: Destroy connection"
    pub const NET_DESTROY_CONNECTION: usize = 0x1BD;
    /// "NET: DPCA"
    pub const NET_DPCA: usize = 0x1D5;
    /// "NET: Find sessions"
    pub const NET_FIND_SESSIONS: usize = 0x1DF;
    /// "NET: Host"
    pub const NET_HOST: usize = 0x1F2;
    /// "NET: Open"
    pub const NET_OPEN: usize = 0x1FC;
    /// "NET: Host complete"
    pub const NET_HOST_COMPLETE: usize = 0x206;
    /// "NET: Join session"
    pub const NET_JOIN_SESSION: usize = 0x219;
    /// "NET: Join complete"
    pub const NET_JOIN_COMPLETE: usize = 0x22B;
    /// "NET: Disable new players"
    pub const NET_DISABLE_NEW_PLAYERS: usize = 0x23E;
    /// "NET: Create dplobby"
    pub const NET_CREATE_DLOBBY: usize = 0x272;
    /// "NET: Create player"
    pub const NET_CREATE_PLAYER: usize = 0x286;
}

impl ViewerPanelBlock {
    /// Whether the developer's keys work in the viewer (1): an unaligned dword in the tail.
    pub fn developer_keys(&self) -> u32 {
        self.tail_read(DEVELOPER_KEYS_AT)
    }

    /// Sets the developer's keys flag (the `wrreplayenable` setting writes it).
    pub fn set_developer_keys(&self, value: u32) {
        self.tail_write(DEVELOPER_KEYS_AT, value);
    }

    /// Pictures a second, 1 .. 100: an unaligned dword in the tail.
    pub fn picture_rate(&self) -> u32 {
        self.tail_read(PICTURE_RATE_AT)
    }

    pub fn set_picture_rate(&self, value: u32) {
        self.tail_write(PICTURE_RATE_AT, value);
    }

    /// Ms a picture: an unaligned dword in the tail.
    pub fn picture_time(&self) -> u32 {
        self.tail_read(PICTURE_TIME_AT)
    }

    pub fn set_picture_time(&self, value: u32) {
        self.tail_write(PICTURE_TIME_AT, value);
    }

    /// The text id of camera `camera` (47 of them; a number past them reads on in the block).
    pub fn camera_name(&self, camera: usize) -> u16 {
        // The names are the block's first bytes.
        crate::memory::cells::load_value::<u16>(self, 2 * camera)
            .unwrap_or_else(|e| crate::fatal::raise::end("a camera's name", e))
    }

    /// The rectangle of the picture `n` in switch table `table` (0 `switch_pictures_a`, 1
    /// `switch_pictures_b`, 2 `switch_pictures_c`; the tables name their rectangles by image
    /// address). The three tables are one run of nine names, which an `n` past a table's end reads
    /// on into; past the run, or a name off the panel, is refused.
    pub fn switch_picture(
        &self,
        table: usize,
        n: u32,
    ) -> Result<[u32; 4], crate::errors::BadInput> {
        let run: [&PlainDword; 9] = [
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

    /// The rectangle (four dwords) the image address `ida` in the panel names; an address off the
    /// panel is refused.
    pub fn rectangle(&self, ida: usize) -> Result<[u32; 4], crate::errors::BadInput> {
        if !(START..END).contains(&ida) {
            return Err(crate::errors::BadInput(
                "a switch picture's name off the panel",
            ));
        }
        crate::memory::cells::load_value(self, ida - START)
    }

    /// The dword at byte `at` of the tail: unaligned, so made of the two cells it spans.
    fn tail_read(&self, at: usize) -> u32 {
        let (n, shift) = Self::tail_cells(at);
        let low = self.untyped_tail[n].get();
        match shift {
            0 => low,
            _ => (low >> shift) | (self.untyped_tail[n + 1].get() << (32 - shift)),
        }
    }

    /// `tail_read`'s write: the bytes of the two cells it spans that the dword covers.
    fn tail_write(&self, at: usize, value: u32) {
        let (n, shift) = Self::tail_cells(at);
        if shift == 0 {
            self.untyped_tail[n].set(value);
            return;
        }
        let (low, high) = (&self.untyped_tail[n], &self.untyped_tail[n + 1]);
        low.set(low.get() & !(u32::MAX << shift) | value << shift);
        high.set(high.get() & (u32::MAX << shift) | value >> (32 - shift));
    }

    /// The tail's cell holding byte `at` and the dword's bit shift in it.
    fn tail_cells(at: usize) -> (usize, u32) {
        assert!(at < 4 * 166);
        (at / 4, 8 * (at % 4) as u32)
    }
}

/// Where the viewer's three settings stand in the tail (bytes from its start, unaligned).
const DEVELOPER_KEYS_AT: usize = 3;
const PICTURE_RATE_AT: usize = 7;
const PICTURE_TIME_AT: usize = 11;

/// The image address of the panel's first byte.
pub const START: usize = 0x5C291C;
/// The image address of the panel's end (the marks' settings follow).
pub const END: usize = 0x5C2F84;

const _: () = assert!(END - START == core::mem::size_of::<ViewerPanelBlock>());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tail_dwords_read_and_write_unaligned() {
        let _g = crate::test_support::test_lock();
        let v = &*VIEWER_PANEL;
        let kept: Vec<u32> = v
            .untyped_tail
            .iter()
            .map(crate::memory::session::PlainDword::get)
            .collect();
        for (k, at) in [0, 3, 7, 11].into_iter().enumerate() {
            for (n, c) in v.untyped_tail.iter().enumerate() {
                c.set(0xA5A5_0000 | n as u32);
            }
            let tail_bytes = || {
                let mut bytes = vec![0; 24];
                crate::memory::cells::load_cells(&v.untyped_tail, 0, &mut bytes).unwrap();
                bytes
            };
            let before = tail_bytes();
            let value = 0x1122_3344 + k as u32;
            v.tail_write(at, value);
            let after = tail_bytes();
            let mut want = before.clone();
            want[at..at + 4].copy_from_slice(&value.to_le_bytes());
            assert_eq!(after, want, "{at}");
            assert_eq!(v.tail_read(at), value);
        }
        for (c, k) in v.untyped_tail.iter().zip(kept) {
            c.set(k);
        }
    }

    #[test]
    fn the_switch_tables_name_their_pictures() {
        let v = &*VIEWER_PANEL;
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
            assert_eq!(
                v.switch_picture(table, n as u32).unwrap(),
                v.rectangle(cell.get() as usize).unwrap()
            );
        }
        assert!(v.switch_picture(0, 9).is_err());
        assert!(v.switch_picture(3, 0).is_err());
    }
}
