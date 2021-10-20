use std::env;
use std::io;
use std::io::BufWriter;
use std::path::Path;

use anyhow::{Context, Result};

use ecat::app;
use ecat::config;
use ecat::file;

fn main() -> Result<()> {
    let config = config::parse_arg()?;

    let isatty: bool = atty::is(atty::Stream::Stdout);
    let color_flag: bool = config.color_when.mix_isatty_to_color_flag(isatty);

    let mut colorizer = app::Colorizer::new();
    let mut config_filepath_list = vec![
        Path::new("./config.yaml").to_path_buf(),
        dirs::home_dir().unwrap().join(".config/ecat/config.yaml"),
    ];
    if config.config_file.is_empty() {
        config_filepath_list.insert(0, Path::new(&config.config_file).to_path_buf());
    }
    for filepath in config_filepath_list.iter() {
        if Path::new(filepath).exists() {
            colorizer.load_config_file(filepath.to_str().unwrap())?;
            break;
        }
    }

    colorizer.setup();
    let line_parse_func =
        |outfile: &mut dyn std::io::Write, nr: i32, s: &String| -> Result<bool, io::Error> {
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
                outfile.write_all(prefix.to_string().as_bytes())?;
                if config.line_number {
                    outfile.write_all((format!("{:>6} ", nr)).as_bytes())?;
                }
                if color_flag {
                    let output = colorizer.colorize(s);
                    outfile.write_all(output.as_bytes())?;
                } else {
                    outfile.write_all(s.as_bytes())?;
                }
                outfile.write_all(suffix.to_string().as_bytes())?;
            }
            // NOTE: skip rest of the file
            if config.base_line > 0 && nr == config.base_line + config.line_context {
                return Ok(false);
            }
            Ok(true)
        };

    let mut writer = BufWriter::new(io::stdout());
    config.files.iter().try_for_each(|filename| -> Result<()> {
        let stdin = std::io::stdin();
        let mut reader = file::Input::console_or_file(&stdin, filename).with_context(|| {
            format!(
                "while opening file '{}' at {}",
                filename,
                env::current_dir().unwrap().to_string_lossy()
            )
        })?;
        file::write_lines(&mut reader, &mut writer, line_parse_func)?;
        Ok(())
    })?;
    Ok(())
}
