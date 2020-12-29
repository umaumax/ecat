use std::fs::File;
use std::io;
use std::io::BufReader;

use anyhow::Result;

pub fn get_buf_reader(file: &str) -> BufReader<Box<dyn std::io::Read>> {
    let read: Box<dyn std::io::Read> = match file {
        "-" => Box::new(io::stdin()),
        _ => Box::new(File::open(file).expect(&(format!("Error opening {} file", file)))),
    };
    BufReader::new(read)
}

pub fn get_buf_reader_safe(file: &str) -> Result<BufReader<Box<dyn std::io::Read>>> {
    let reader: Box<dyn std::io::Read> = match file {
        "-" => Box::new(io::stdin()),
        _ => {
            if std::path::Path::new(file).is_dir() {
                return Err(anyhow!("{} is a directory, not a file", file));
            }
            Box::new(File::open(file)?)
        }
    };
    Ok(BufReader::new(reader))
}
