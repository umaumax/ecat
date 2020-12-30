use std::env;
use std::error::Error;
use std::fmt;
use std::io;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::process;

use anyhow::{Context, Result};

use ecat::app;
use ecat::config;
use ecat::file;

fn main() -> Result<()> {
    let config = config::parse_arg().unwrap();

    let isatty: bool = atty::is(atty::Stream::Stdout);
    let color_flag: bool = config.color_when.mix_isatty_to_color_flag(isatty);

    let mut config_filepath_list = vec![
        Path::new("./config.yaml").to_path_buf(),
        dirs::home_dir().unwrap().join(".config/ecat/config.yaml"),
    ];
    if config.config_file.len() > 0 {
        config_filepath_list.insert(0, Path::new(&config.config_file).to_path_buf());
    }
    let mut colorizer = app::Colorizer::new();

    #[derive(Debug)]
    struct SkipError;
    impl fmt::Display for SkipError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "SkipError")
        }
    }
    impl Error for SkipError {}

    config_filepath_list
        .iter()
        .try_for_each(|filepath| -> Result<(), Box<dyn std::error::Error>> {
            if Path::new(filepath).exists() {
                colorizer.load_config_file(filepath.to_str().unwrap())?;
                return Err(Box::new(SkipError {}));
            }
            Ok(())
        })
        .unwrap_or_else(|e| match e {
            e if e.downcast_ref::<SkipError>().is_some() => {}
            _ => {
                eprintln!("Problem while reading files: {}", e);
                process::exit(1);
            }
        });

    colorizer.setup();
    let line_parse_func = |outfile: &mut Box<dyn std::io::Write>, nr: i32, s: &String| -> bool {
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
            outfile.write((format!("{}", prefix)).as_bytes()).unwrap();
            if config.line_number {
                outfile.write((format!("{:>6} ", nr)).as_bytes()).unwrap();
            }
            if color_flag {
                let output = colorizer.colorize(s);
                outfile.write(output.as_bytes()).unwrap();
            } else {
                outfile.write(s.as_bytes()).unwrap();
            }
            outfile.write((format!("{}", suffix)).as_bytes()).unwrap();
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
            let mut reader = file::get_buf_reader_safe(filename).with_context(|| {
                format!(
                    "while opening file '{}' at {}",
                    filename,
                    env::current_dir().unwrap().to_string_lossy()
                )
            })?;
            let mut w: Box<dyn std::io::Write> = Box::new(BufWriter::new(io::stdout()));
            file::write_lines(&mut reader, &mut w, line_parse_func)?;
            Ok(())
        })
        .unwrap_or_else(|err| {
            eprintln!("Problem while reading files: {}", err);
            process::exit(1);
        });
    Ok(())
}
