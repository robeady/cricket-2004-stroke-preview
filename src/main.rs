use anyhow::Result;
use serde_derive::Deserialize;
use std::fs::File;
use std::io::Read;
use window::render_app;

mod data;
mod pitch_canvas;
mod strokes;
mod window;

fn main() -> Result<()> {
    let files = load_default_files()?;
    Ok(render_app(files)?)
}

#[derive(Deserialize)]
pub struct Files {
    cfg_file: String,
    list_file: String,
}

/// loads the default file paths from stoke_preview.toml or returns empty strings if that file does not exist
fn load_default_files() -> anyhow::Result<Files> {
    match File::open("stroke_preview.toml") {
        Ok(mut file) => {
            let mut string =
                String::with_capacity(file.metadata().map(|m| m.len() as usize + 1).unwrap_or(0));
            file.read_to_string(&mut string)?;
            Ok(toml::from_str(&string)?)
        }
        Err(_) => Ok(Files { cfg_file: "AI.cfg".to_string(), list_file: "List.txt".to_string() }),
    }
}
