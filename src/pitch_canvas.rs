use crate::strokes::{Stroke, StrokeTiming};
use std::cmp::min;
use std::f64::consts::TAU;
use winapi::shared::windef::{HBRUSH, HDC, RECT};
use winapi::um::wingdi::{
    CreateSolidBrush, Ellipse, GetStockObject, Pie, SelectObject, NULL_PEN, RGB, WHITE_PEN,
};
use winapi::um::winuser::{FillRect, GetSysColorBrush, COLOR_MENU};

pub struct PitchPainter {
    background: HBRUSH,

    green: HBRUSH,
    dark_green: HBRUSH,

    stroke_min: HBRUSH,
    stroke_max: HBRUSH,
}

impl PitchPainter {
    pub fn new() -> PitchPainter {
        unsafe {
            PitchPainter {
                // FIXME: native-windows-gui sets the wrong background color and we can't change it
                background: GetSysColorBrush(COLOR_MENU),
                green: CreateSolidBrush(RGB(0, 190, 0)),
                dark_green: CreateSolidBrush(RGB(0, 150, 0)),
                stroke_min: CreateSolidBrush(RGB(250, 100, 50)),
                stroke_max: CreateSolidBrush(RGB(250, 250, 30)),
            }
        }
    }

    pub fn paint(
        &self,
        paint: &nwg::PaintData,
        stroke: Option<&Stroke>,
        selected_timing: usize,
        selected_6hit: bool,
    ) {
        let ps = paint.begin_paint();

        unsafe {
            let hdc = ps.hdc;
            let rc = &ps.rcPaint;

            FillRect(hdc, rc, self.background as _);

            // pitch

            let padding = 5;
            SelectObject(hdc, self.green as _);
            SelectObject(hdc, GetStockObject(WHITE_PEN as _));
            // make sure it's round
            let height = rc.bottom - rc.top;
            let width = rc.right - rc.left;
            let radius = min(height, width) / 2 - padding;
            let centre_x = (rc.left + rc.right) / 2;
            let centre_y = (rc.top + rc.bottom) / 2;
            let bounds = RECT {
                left: centre_x - radius,
                top: centre_y - radius,
                right: centre_x + radius,
                bottom: centre_y + radius,
            };
            Ellipse(hdc, bounds.left, bounds.top, bounds.right, bounds.bottom);

            // strokes

            SelectObject(hdc, GetStockObject(NULL_PEN as _));

            if let Some(stroke) = stroke {
                let timings =
                    if selected_6hit { &stroke.timings_6hit } else { &stroke.timings_normal };
                for i in 0..5 {
                    if selected_timing != i {
                        self.paint_stroke_segment(hdc, &timings[i], bounds, false);
                    }
                }
                self.paint_stroke_segment(hdc, &timings[selected_timing], bounds, true);
            }
        }

        paint.end_paint(&ps);
    }

    fn paint_stroke_segment(
        &self,
        hdc: HDC,
        stroke: &StrokeTiming,
        bounds: RECT,
        highlighted: bool,
    ) {
        let centre_x = (bounds.left + bounds.right) as f64 / 2.0;
        let centre_y = (bounds.top + bounds.bottom) as f64 / 2.0;

        let pitch_radius = (bounds.right - bounds.left) as f64 / 2.0; // it's a circle

        // angle is in radians anticlockwise 0 being directly behind the batsman
        let angle = (stroke.direction as f64 - 60_000.0) / 269_070_000.0 * TAU;

        let min_angle = (angle - stroke.direction_area / 269_070_000.0 * TAU) % TAU;
        let max_angle = (angle + stroke.direction_area / 269_070_000.0 * TAU) % TAU;

        let min_radial_intercept_x = centre_x - 100.0 * min_angle.sin();
        let min_radial_intercept_y = centre_y - 100.0 * min_angle.cos();

        let max_radial_intercept_x = centre_x - 100.0 * max_angle.sin();
        let max_radial_intercept_y = centre_y - 100.0 * max_angle.cos();

        for &(radius_unscaled, brush) in [
            (stroke.power + stroke.power_area, self.stroke_max),
            (stroke.power - stroke.power_area, self.stroke_min),
        ]
        .iter()
        {
            let shot_radius = pitch_radius * radius_unscaled / 4_500_000.0;

            unsafe {
                SelectObject(hdc, if highlighted { brush } else { self.dark_green } as _);

                Pie(
                    hdc,
                    (centre_x - shot_radius) as i32,
                    (centre_y - shot_radius) as i32,
                    (centre_x + shot_radius) as i32,
                    (centre_y + shot_radius) as i32,
                    min_radial_intercept_x as i32,
                    min_radial_intercept_y as i32,
                    max_radial_intercept_x as i32,
                    max_radial_intercept_y as i32,
                );
            }
        }
    }
}
