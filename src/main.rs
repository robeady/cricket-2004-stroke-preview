use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
};

use indexmap::IndexSet;
use window::{render_ui, UiData};

mod pitch_canvas;
mod strokes;
mod window;

fn main() -> anyhow::Result<()> {
    let list = read_list_file(r"C:\Users\Rob\OneDrive\Projects\Cricket 2004\AI configs\List.txt");
    let cfgs = read_cfg_file(
        r"C:\Users\Rob\OneDrive\Projects\Cricket 2004\AI configs\My AI Configs Newer.cfg",
    );
    let cfgs: IndexSet<_> = cfgs.iter().collect();
    let mut ai_cfg_names: Vec<_> = list
        .into_iter()
        .skip_while(|item| item.starts_with("Screen"))
        .take_while(|item| !item.starts_with("Fast"))
        .collect();
    sort_ai_cfg_names(&mut ai_cfg_names);
    if ai_cfg_names.len() != cfgs.len() {
        panic!("wrong length, {} AI cfg names but {} cfgs", ai_cfg_names.len(), cfgs.len());
    }

    render_ui(UiData { stroke_cfg_names: ai_cfg_names, strokes: HashMap::new() })
}

fn sort_ai_cfg_names(ai_cfg_names: &mut Vec<String>) {
    ai_cfg_names.sort_by_cached_key(|name| {
        let parts = name.split(",").collect::<Vec<_>>();
        parts[1].parse::<i64>().unwrap()
    });
    print!("{:#?}", ai_cfg_names);
}

fn read_cfg_file(path: &str) -> Vec<String> {
    let file = File::open(path).expect("cfg file not found");
    let mut buf = BufReader::new(file);
    let bytes_of_non_strokes = 25696336 - 25665536;
    buf.seek(SeekFrom::Start(bytes_of_non_strokes)).unwrap();
    buf.split(0)
        .map(|r| r.expect("error reading cfg file"))
        .filter(|s| s.len() > 20 /* arbitrary cut off because of weird bytes */)
        .map(|s| s.iter().map(|&c| c as char).collect::<String>())
        .filter(|s| !s.contains("September 5th, 2001") /* filter out weird notice */)
        .collect()
}

fn read_list_file(path: &str) -> Vec<String> {
    let file = File::open(path).expect("list file not found");
    let buf = BufReader::new(file);
    buf.lines()
        .enumerate()
        .map(|(i, l)| l.unwrap_or_else(|e| panic!("error parsing line {}: {}", i, e)))
        .filter(|line| !line.starts_with("//"))
        .collect()
}
