use std::cmp::min;
use std::f64::consts::TAU;

use winapi::{
    shared::windef::{HBRUSH, HDC, HPEN, RECT},
    um::{
        wingdi::{
            CreatePen, CreateSolidBrush, Ellipse, GetStockObject, Pie, SelectObject, NULL_PEN,
            PS_SOLID, RGB, WHITE_PEN,
        },
        winuser::{FillRect, GetSysColorBrush, COLOR_MENU},
    },
};

use crate::strokes::{Stroke, StrokeTiming};

pub struct PitchPainter {
    background: HBRUSH,
    border: HBRUSH,
    pen: HPEN,
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
                border: CreateSolidBrush(RGB(100, 100, 255)),
                pen: CreatePen(PS_SOLID as _, 2, RGB(20, 20, 20)),
                green: CreateSolidBrush(RGB(0, 190, 0)),
                dark_green: CreateSolidBrush(RGB(0, 150, 0)),
                stroke_min: CreateSolidBrush(RGB(250, 100, 50)),
                stroke_max: CreateSolidBrush(RGB(250, 250, 30)),
            }
        }
    }

    pub fn paint(&self, paint: &nwg::PaintData, stroke: Option<&Stroke>, selected_timing: usize) {
        let ps = paint.begin_paint();

        unsafe {
            let hdc = ps.hdc;
            let rc = &ps.rcPaint;

            FillRect(hdc, rc, self.background as _);

            // pitch
            let pad = 5;
            SelectObject(hdc, self.green as _);
            SelectObject(hdc, GetStockObject(WHITE_PEN as _));
            // make sure it's round
            let height = rc.bottom - rc.top;
            let width = rc.right - rc.left;
            let radius = min(height, width) / 2 - pad;
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
                for i in 0..5 {
                    if selected_timing != i {
                        self.paint_stroke_segment(hdc, &stroke.timings_normal[i], bounds, false);
                    }
                }
                self.paint_stroke_segment(
                    hdc,
                    &stroke.timings_normal[selected_timing],
                    bounds,
                    true,
                );
            }

            // FillRect(hdc, rc, self.background as _);
            // FrameRect(hdc, rc, self.border as _);

            // SelectObject(hdc, self.pen as _);
            // SelectObject(hdc, self.yellow as _);
            // Ellipse(hdc, rc.left + 20, rc.top + 20, rc.right - 20, rc.bottom - 20);

            // SelectObject(hdc, self.white as _);
            // Ellipse(hdc, 60, 60, 130, 130);
            // Ellipse(hdc, 150, 60, 220, 130);

            // SelectObject(hdc, self.black as _);

            // Ellipse(hdc, 80, 80, 110, 110);
            // Ellipse(hdc, 170, 80, 200, 110);

            // SelectObject(hdc, self.red as _);
            // let pts = &[P { x: 60, y: 150 }, P { x: 220, y: 150 }, P { x: 140, y: 220 }];
            // Polygon(hdc, pts.as_ptr(), pts.len() as _);
        }

        paint.end_paint(&ps);
    }

    /// brush should be semi-transparent as we will draw twice, once for min power and once for max
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

        let min_angle = dbg!((angle - stroke.direction_area / 269_070_000.0 * TAU) % TAU);
        let max_angle = dbg!((angle + stroke.direction_area / 269_070_000.0 * TAU) % TAU);

        let min_radial_intercept_x = dbg!(centre_x - 100.0 * min_angle.sin());
        let min_radial_intercept_y = dbg!(centre_y - 100.0 * min_angle.cos());

        let max_radial_intercept_x = dbg!(centre_x - 100.0 * max_angle.sin());
        let max_radial_intercept_y = dbg!(centre_y - 100.0 * max_angle.cos());

        for &(radius_unscaled, brush) in [
            (stroke.power + stroke.power_area, self.stroke_max),
            (stroke.power - stroke.power_area, self.stroke_min),
        ]
        .iter()
        {
            let shot_radius = dbg!(pitch_radius * radius_unscaled / 4_500_000.0);

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
            // LineTo(hdc, nXEnd, nYEnd);
        }
    }
}
