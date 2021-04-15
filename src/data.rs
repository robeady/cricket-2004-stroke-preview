use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

use anyhow::Context;

use crate::Files;

pub struct CfgData {
    pub cfg_items: Vec<(String, i64)>,
    pub cfg_contents: Vec<u8>,
}

pub fn load_cfg_data(files: &Files) -> anyhow::Result<CfgData> {
    let mut cfg_items: Vec<_> = read_list_file(&files.list_file)?
        .map(|s| {
            let parts = s.split(",").collect::<Vec<_>>();
            (parts[0].to_string(), parts[1].parse::<i64>().unwrap())
        })
        // https://www.planetcricket.org/forums/threads/config-editor-v3.8697/post-130389
        .filter(|(_, offset)| (558891008..=559079424).contains(offset))
        .collect();
    cfg_items.sort_by_key(|(_, offset)| *offset);

    let cfg_contents = read_strokes_from_ai_cfg_file(&files.cfg_file)?;

    Ok(CfgData { cfg_items, cfg_contents })
}

fn read_strokes_from_ai_cfg_file(path: &str) -> anyhow::Result<Vec<u8>> {
    let file = File::open(path).context("could not open cfg file")?;

    // offset found experimentally
    let bytes_of_non_strokes: u64 = 0x7c60;
    let buffer_size = file.metadata().map(|m| m.len() + 1 - bytes_of_non_strokes).unwrap_or(0);

    let mut buf = BufReader::new(file);
    buf.seek(SeekFrom::Start(bytes_of_non_strokes)).unwrap();
    let mut destination = Vec::with_capacity(buffer_size as usize);
    buf.read_to_end(&mut destination)?;
    Ok(destination)
}

fn read_list_file(path: &str) -> anyhow::Result<impl Iterator<Item = String>> {
    let file = File::open(path).context("could not open list file")?;
    let buf = BufReader::new(file);
    Ok(buf
        .lines()
        .enumerate()
        .map(|(i, l)| l.unwrap_or_else(|e| panic!("error parsing line {}: {}", i, e)))
        .filter(|line| !line.starts_with("//")))
}
