use regex::Regex;
use std::io::{BufWriter, Write};

pub struct ColorPatternMap {
    pub name: String,
    pub pattern: String,
    pub regex_pattern: regex::Regex,
    pub color: ansi_term::Colour,
}

impl ColorPatternMap {
    pub fn new(name: &str, pattern: &str, color: ansi_term::Colour) -> Self {
        let pattern = format!("({})|$", pattern);
        ColorPatternMap {
            name: name.to_string(),
            pattern: pattern.to_string(),
            regex_pattern: Regex::new(&pattern).unwrap(),
            color: color,
        }
    }
}
pub fn process_color_pattern_maps(s: &str, patterns: &[ColorPatternMap]) -> String {
    if patterns.len() == 0 {
        return String::from("");
    }
    let mut buf = vec![];

    let color_pattern_map = &patterns[0];
    let re = &color_pattern_map.regex_pattern;
    {
        let mut f = BufWriter::new(&mut buf);

        let mut offset = 0;
        let words = re.find_iter(s);
        for mat in words {
            let input = s;
            let word = &input[offset..mat.start()];
            let target_word = mat.as_str();
            offset = mat.end();

            if patterns.len() > 1 {
                let ret = process_color_pattern_maps(word, &patterns[1..]);
                f.write(ret.as_bytes()).unwrap();
            } else {
                // nothing match
                f.write(word.as_bytes()).unwrap();
            }

            if target_word.len() > 0 {
                let prefix = color_pattern_map.color.prefix().to_string();
                let suffix = color_pattern_map.color.suffix().to_string();
                f.write(prefix.as_bytes()).unwrap();
                f.write(target_word.as_bytes()).unwrap();
                f.write(suffix.as_bytes()).unwrap();
            }
        }
    }
    let output = std::str::from_utf8(&buf).unwrap().to_string();
    return output;
}

pub struct Colorizer {
    pub color_pattern_maps: Vec<ColorPatternMap>,
}

impl Colorizer {
    pub fn new() -> Self {
        struct ColorPatternMapTemplate {
            name: String,
            patterns: Vec<String>,
            color: ansi_term::Colour,
        };
        let color_pattern_map_data_list: Vec<ColorPatternMapTemplate> = vec![
            ColorPatternMapTemplate {
                name: String::from("ip_addr"),
                patterns: vec![
                    r"[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}", // TODO: add ipv6
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                color: ansi_term::Colour::Fixed(228),
            },
            ColorPatternMapTemplate {
                name: String::from("filepath"),
                patterns: vec![r"(\.[0-9a-zA-Z~\-_/.]+)|([0-9a-zA-Z~\-_/.]+\.[0-9a-zA-Z~\-_]+)"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                color: ansi_term::Colour::Fixed(78),
            },
            ColorPatternMapTemplate {
                name: String::from("warnings"),
                patterns: vec!["tmp", "fix"].into_iter().map(String::from).collect(),
                color: ansi_term::Colour::Fixed(209),
            },
            ColorPatternMapTemplate {
                name: String::from("hex"),
                patterns: vec![r"0x[0-9a-fA-F]+"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                color: ansi_term::Colour::White,
            },
            ColorPatternMapTemplate {
                name: String::from("word"), // for preventing match en0 to "en" and "0" after "number" match
                patterns: vec![r"(([a-zA-Z_]+[0-9]+)|([0-9]+[a-zA-Z_]+))[0-9a-zA-Z_]*"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                color: ansi_term::Colour::Fixed(32), // blue
            },
            ColorPatternMapTemplate {
                name: String::from("number"),
                patterns: vec![r"[0-9]+", r"0x[0-9a-fA-F]+"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                color: ansi_term::Colour::White,
            },
            ColorPatternMapTemplate {
                name: String::from("default"),
                patterns: vec![r".+"].into_iter().map(String::from).collect(),
                color: ansi_term::Colour::Fixed(32), // blue
            },
        ];
        let mut color_pattern_maps: Vec<ColorPatternMap> = vec![];
        for color_pattern_map_data in color_pattern_map_data_list {
            let pattern = color_pattern_map_data.patterns.join("|");
            let m = ColorPatternMap::new(
                &color_pattern_map_data.name,
                &pattern,
                color_pattern_map_data.color,
            );
            color_pattern_maps.push(m);
        }

        let colorizer = Colorizer { color_pattern_maps };
        return colorizer;
    }
    pub fn colorize(&self, s: &str) -> String {
        return process_color_pattern_maps(s, &self.color_pattern_maps);
    }
}
