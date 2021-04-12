use std::cmp::min;
use std::f64::consts::TAU;

use winapi::{
    shared::windef::{HBRUSH, HDC, HPEN, POINT, RECT},
    um::{
        wingdi::{
            CreatePen, CreateSolidBrush, Ellipse, LineTo, MoveToEx, Pie, SelectObject, PS_SOLID,
            RGB,
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
    stroke_vearly: HBRUSH,
    stroke_early: HBRUSH,
    stroke_ideal: HBRUSH,
    stroke_late: HBRUSH,
    stroke_vlate: HBRUSH,
}

impl PitchPainter {
    pub fn new() -> PitchPainter {
        unsafe {
            PitchPainter {
                // FIXME: native-windows-gui sets the wrong background color and we can't change it
                background: GetSysColorBrush(COLOR_MENU),
                border: CreateSolidBrush(RGB(100, 100, 255)),
                pen: CreatePen(PS_SOLID as _, 2, RGB(20, 20, 20)),
                green: CreateSolidBrush(RGB(0, 150, 0)),
                stroke_vearly: CreateSolidBrush(RGB(255, 122, 0)),
                stroke_early: CreateSolidBrush(RGB(230, 30, 10)),
                stroke_ideal: CreateSolidBrush(RGB(240, 0, 250)),
                stroke_late: CreateSolidBrush(RGB(60, 20, 230)),
                stroke_vlate: CreateSolidBrush(RGB(0, 120, 250)),
            }
        }
    }

    pub fn paint(&self, paint: &nwg::PaintData, stroke: Option<&Stroke>) {
        let ps = paint.begin_paint();

        unsafe {
            let hdc = ps.hdc;
            let rc = &ps.rcPaint;

            FillRect(hdc, rc, self.background as _);

            // pitch
            let pad = 5;
            SelectObject(hdc, self.green as _);
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

            if let Some(stroke) = stroke {
                let brushes = [
                    self.stroke_vearly,
                    self.stroke_early,
                    self.stroke_ideal,
                    self.stroke_late,
                    self.stroke_vlate,
                ];

                //for i in 0..5 {
                paint_stroke_segment(hdc, &stroke.timings_normal[2], bounds);
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
}

/// brush should be semi-transparent as we will draw twice, once for min power and once for max
fn paint_stroke_segment(hdc: HDC, stroke: &StrokeTiming, bounds: RECT) {
    for &radius in [stroke.power - stroke.power_area, stroke.power + stroke.power_area].iter() {
        let radius = radius as f64;

        // angle is in radians anticlockwise 0 being directly behind the batsman
        let angle = (stroke.direction as f64 - 60_000.0) / 269_070_000.0 * TAU;

        let min_angle = angle - stroke.direction_area / 269_070_000.0 * TAU;
        let max_angle = angle + stroke.direction_area / 269_070_000.0 * TAU;

        let centre_x = (bounds.left + bounds.right) as f64 / 2.0;
        let centre_y = (bounds.top + bounds.bottom) as f64 / 2.0;

        let min_radial_endpoint_x = centre_x - radius * min_angle.sin();
        let min_radial_endpoint_y = centre_y - radius * min_angle.cos();

        let max_radial_endpoint_x = centre_x - radius * max_angle.sin();
        let max_radial_endpoint_y = centre_y - radius * max_angle.cos();

        unsafe {
            Pie(
                hdc,
                bounds.left,
                bounds.top,
                bounds.right,
                bounds.bottom,
                min_radial_endpoint_x as i32,
                min_radial_endpoint_y as i32,
                max_radial_endpoint_x as i32,
                max_radial_endpoint_y as i32,
            );
        }
        // LineTo(hdc, nXEnd, nYEnd);
    }
}
