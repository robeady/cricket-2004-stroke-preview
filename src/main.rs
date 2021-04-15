use anyhow::Result;
use window::render_app;

mod data;
mod pitch_canvas;
mod strokes;
mod window;

fn main() -> Result<()> {
    Ok(render_app()?)
}
