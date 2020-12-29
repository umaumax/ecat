use std::env;
use std::io;
use std::io::{BufWriter, Write};
use std::ops::Fn;
use std::process;

use anyhow::{Context, Result};

use ecat::app::colorize;
use ecat::config;
use ecat::file::get_buf_reader_safe;

fn main() -> Result<()> {
    let config = config::parse_arg().unwrap();

    let isatty: bool = atty::is(atty::Stream::Stdout);
    let color_flag: bool = config.color_when.mix_isatty_to_color_flag(isatty);

    let f = |outfile: &mut Box<dyn std::io::Write>, nr: i32, s: &String| -> bool {
        let output_flag = config.base_line <= 0
            || config.base_line - config.line_context <= nr
                && nr <= config.base_line + config.line_context;
        if output_flag {
            let mut prefix = "";
            let mut suffix = "";
            if config.base_line == nr && color_flag {
                prefix = "\x1b[32m"; // NOTE: green
                suffix = "\x1b[m";
            }
            {
                outfile
                    .write((format!("{}{:>6} ", prefix, nr)).as_bytes())
                    .unwrap();
                let output = colorize(s);
                outfile.write(output.as_bytes()).unwrap();
                outfile.write((format!("{}", suffix)).as_bytes()).unwrap();
            }
        }
        // NOTE: skip rest of the file
        if config.base_line > 0 && nr == config.base_line + config.line_context {
            return false;
        }
        true
    };

    config
        .files
        .iter()
        .try_for_each(|filename| -> Result<()> {
            let mut reader = get_buf_reader_safe(filename).with_context(|| {
                format!(
                    "while opening file '{}' at {}",
                    filename,
                    env::current_dir().unwrap().to_string_lossy()
                )
            })?;
            let mut w: Box<dyn std::io::Write> = Box::new(BufWriter::new(io::stdout()));
            write_lines(&mut reader, &mut w, f)?;
            Ok(())
        })
        .unwrap_or_else(|err| {
            eprintln!("Problem while reading files: {}", err);
            process::exit(1);
        });
    Ok(())
}

fn write_lines<F>(
    r: &mut std::io::BufRead,
    w: &mut Box<dyn std::io::Write>,
    f: F,
) -> Result<(), io::Error>
where
    F: Fn(&mut Box<dyn std::io::Write>, i32, &String) -> bool,
{
    let mut s = String::new();
    let mut nr = 1;
    loop {
        match r.read_line(&mut s) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let ret = f(w, nr, &s);
                s.clear();
                if !ret {
                    break;
                }
            }
            Err(err) => return Err(err),
        }
        nr += 1;
    }
    Ok(())
}
