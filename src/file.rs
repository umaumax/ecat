use std::fs::File;
use std::io;
use std::io::BufReader;
use std::time::Instant;

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

pub fn write_lines<F>(
    r: &mut dyn std::io::BufRead,
    w: &mut dyn std::io::Write,
    f: F,
) -> Result<(), io::Error>
where
    F: Fn(&mut dyn std::io::Write, i32, &String) -> bool,
{
    let mut s = String::new();
    let mut nr = 1;

    let flush_timeout_th = 200;
    let mut start = Instant::now();
    loop {
        match r.read_line(&mut s) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let ret = f(w, nr, &s);
                s.clear();
                if !ret {
                    break;
                }
                let now = start.elapsed();
                if now.as_millis() >= flush_timeout_th {
                    w.flush().unwrap();
                    start = Instant::now();
                }
            }
            Err(err) => return Err(err),
        }
        nr += 1;
    }
    Ok(())
}
