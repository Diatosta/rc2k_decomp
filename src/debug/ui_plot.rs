//! A plot on ImGui's window draw list, for the debug UI's graphs.

use dear_imgui_rs::Ui;

/// A rectangle of the window taking values to places.
pub struct Plot {
    at: [f32; 2],
    size: [f32; 2],
    x: [f32; 2],
    y: [f32; 2],
}

impl Plot {
    /// The next `size` of the window, showing `x` and `y`.
    pub fn new(ui: &Ui, size: [f32; 2], x: [f32; 2], y: [f32; 2]) -> Self {
        let at = ui.cursor_screen_pos();
        ui.dummy(size);
        let list = ui.get_window_draw_list();
        list.add_rect(
            at,
            [at[0] + size[0], at[1] + size[1]],
            [0.05, 0.05, 0.07, 1.0],
        )
        .filled(true)
        .build();
        Self { at, size, x, y }
    }

    fn place(&self, (x, y): (f32, f32)) -> [f32; 2] {
        [
            self.at[0] + (x - self.x[0]) / (self.x[1] - self.x[0]) * self.size[0],
            self.at[1] + self.size[1] - (y - self.y[0]) / (self.y[1] - self.y[0]) * self.size[1],
        ]
    }

    /// Lines at each `step_x` and `step_y`, with their values; the zero line brighter.
    pub fn grid(&self, ui: &Ui, step_x: f32, step_y: f32) {
        let list = ui.get_window_draw_list();
        let dim = [0.25, 0.25, 0.3, 1.0];
        let mut x = self.x[0];
        while x <= self.x[1] {
            let top = self.place((x, self.y[1]));
            list.add_line(top, self.place((x, self.y[0])), dim).build();
            list.add_text(
                [top[0] + 2.0, self.at[1] + self.size[1] - 14.0],
                [0.6, 0.6, 0.6, 1.0],
                format!("{x:.0}"),
            );
            x += step_x;
        }
        let mut y = (self.y[0] / step_y).ceil() * step_y;
        while y <= self.y[1] {
            let left = self.place((self.x[0], y));
            let colour = if y.abs() < step_y / 2.0 {
                [0.7, 0.7, 0.7, 1.0]
            } else {
                dim
            };
            list.add_line(left, self.place((self.x[1], y)), colour)
                .build();
            list.add_text(
                [left[0] + 2.0, left[1] - 14.0],
                [0.6, 0.6, 0.6, 1.0],
                format!("{y:.2}"),
            );
            y += step_y;
        }
    }

    /// A line between two values.
    pub fn line(&self, ui: &Ui, a: (f32, f32), b: (f32, f32), colour: [f32; 4]) {
        ui.get_window_draw_list()
            .add_line(self.place(a), self.place(b), colour)
            .thickness(1.5)
            .build();
    }

    /// Lines through values.
    pub fn path(&self, ui: &Ui, points: &[(f32, f32)], colour: [f32; 4]) {
        for pair in points.windows(2) {
            self.line(ui, pair[0], pair[1], colour);
        }
    }

    /// Lines across the plot through a value, and a dot on it.
    pub fn cross(&self, ui: &Ui, at: (f32, f32)) {
        let list = ui.get_window_draw_list();
        let white = [1.0, 1.0, 1.0, 0.6];
        list.add_line(
            self.place((at.0, self.y[0])),
            self.place((at.0, self.y[1])),
            white,
        )
        .build();
        list.add_line(
            self.place((self.x[0], at.1)),
            self.place((self.x[1], at.1)),
            white,
        )
        .build();
        list.add_circle(self.place(at), 4.0, [1.0, 1.0, 1.0, 1.0])
            .filled(true)
            .build();
    }
}
