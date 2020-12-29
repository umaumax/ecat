use std::str::FromStr;

use anyhow::{Context, Result};

#[derive(strum_macros::EnumString)]
#[strum(serialize_all = "kebab_case")]
pub enum ColorWhen {
    Always,
    Never,
    Auto,
}

impl ColorWhen {
    pub fn mix_isatty_to_color_flag(&self, isatty: bool) -> bool {
        match self {
            ColorWhen::Always => true,
            ColorWhen::Never => false,
            ColorWhen::Auto => isatty,
        }
    }
}

pub struct Config {
    pub base_line: i32,
    pub line_context: i32,
    pub color_when: ColorWhen,
    pub files: Vec<String>,
}

pub fn parse_arg() -> Result<Config> {
    let matches = build_app().get_matches();
    let base_line = matches
        .value_of("line")
        .unwrap_or("0")
        .parse::<i32>()
        .with_context(|| format!("failed parse --line option"))?;
    let line_context = matches
        .value_of("context")
        .unwrap_or("3")
        .parse::<i32>()
        .with_context(|| format!("failed parse -C, --context option"))?;
    let color_when = ColorWhen::from_str(matches.value_of("color").unwrap_or("auto"))
        .with_context(|| format!("failed parse --color option"))?;
    let mut files: Vec<String> = matches
        .values_of("files")
        .unwrap()
        .map(String::from)
        .collect();
    // NOTE: default input is stdin
    if files.len() == 0 {
        files.push(String::from("-"));
    }
    let config = Config {
        base_line,
        line_context,
        color_when,
        files,
    };
    return Ok(config);
}

pub fn build_app() -> clap::App<'static, 'static> {
    let program = std::env::args()
        .nth(0)
        .and_then(|s| {
            std::path::PathBuf::from(s)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
        })
        .unwrap();

    clap::App::new(program)
        .about("original cat command by rust")
        .version("0.0.1")
        .setting(clap::AppSettings::VersionlessSubcommands)
        .arg(clap::Arg::from_usage(
            "--color=[WHEN] \
            'use markers to highlight the mathing strings; \
            WHEN is [always], [never], or [auto]'",
        ))
        .arg(clap::Arg::from_usage(
            "--line=[NUM] \
            'print taeget line of output context;",
        ))
        .arg(clap::Arg::from_usage(
            "-C --context=[NUM] \
            'print NUM lines of output context;",
        ))
        .arg(
            clap::Arg::with_name("files")
                .help("Sets the input file to use")
                .required(true)
                .multiple(true)
                .index(1),
        )
}
