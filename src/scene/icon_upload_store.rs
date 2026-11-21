//! The icon upload's work record: where a car icon goes on its texture and which piece of the
//! sheet it is, while `scene::upload_car_icon` scales the sheet and blits the piece.

#[derive(Clone, Copy, Debug)]
pub struct IconUpload {
    /// Where on the texture the icon goes.
    pub target_x: u32,
    pub target_y: u32,
    /// The icon's piece of the sheet: x, y, width, height (`blit_components`' rectangle).
    pub piece: [u32; 4],
}
