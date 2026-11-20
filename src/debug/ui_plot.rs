//! A plot on ImGui's window draw list, for the debug UI's graphs.

use dear_imgui_rs::Ui;

/// The most grid lines drawn along one axis.
const MOST_GRID_LINES: f32 = 64.0;

/// The step a grid along `from ..= to` is drawn at: `step`, or a wider one when `step` would
/// make more than `MOST_GRID_LINES` lines; none for a step or a range that is not a finite
/// positive number (the loop over such a range would not end, or would draw millions of lines).
fn grid_step(from: f32, to: f32, step: f32) -> Option<f32> {
    let span = to - from;
    if !(step.is_finite() && step > 0.0 && span.is_finite() && span > 0.0) {
        return None;
    }
    Some(step.max(span / MOST_GRID_LINES))
}

/// A rectangle of the window taking values to places.
pub(super) struct Plot {
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
        let (Some(step_x), Some(step_y)) = (
            grid_step(self.x[0], self.x[1], step_x),
            grid_step(self.y[0], self.y[1], step_y),
        ) else {
            return;
        };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_grid_step_is_kept_when_it_makes_few_lines() {
        assert_eq!(grid_step(0.0, 71.0, 10.0), Some(10.0));
    }

    #[test]
    fn a_grid_step_is_widened_when_it_would_make_too_many_lines() {
        // A range of a million with a step of 0.001 would be a billion lines.
        let step = grid_step(-1.0e6, 10.0, 0.001).unwrap();
        assert!((1.0e6 / step) <= MOST_GRID_LINES + 1.0);
    }

    #[test]
    fn a_grid_over_no_range_or_a_bad_step_is_not_drawn() {
        for (from, to, step) in [
            (0.0, 0.0, 1.0),
            (0.0, f32::INFINITY, 1.0),
            (f32::NEG_INFINITY, 1.0, 1.0),
            (0.0, f32::NAN, 1.0),
            (0.0, 1.0, 0.0),
            (0.0, 1.0, -1.0),
            (0.0, 1.0, f32::INFINITY),
            (0.0, 1.0, f32::NAN),
        ] {
            assert_eq!(grid_step(from, to, step), None, "{from} {to} {step}");
        }
    }
}
