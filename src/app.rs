use regex::Regex;
use std::io::{BufWriter, Write};

use anyhow::Result;

use serde::Deserializer;
use serde::{Deserialize, Serialize};

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
    color_pattern_map_data_list: Vec<ColorPatternMapTemplate>,
    pub color_pattern_maps: Vec<ColorPatternMap>,
}

pub struct ColorSet {
    name: String,
    color: ansi_term::Colour,
}
impl ColorSet {
    pub fn new(name: &str) -> Self {
        let color = match name {
            name if name.starts_with("#") => {
                let r = u8::from_str_radix(&name[1..3], 16).unwrap();
                let g = u8::from_str_radix(&name[3..5], 16).unwrap();
                let b = u8::from_str_radix(&name[5..7], 16).unwrap();
                ansi_term::Color::RGB(r, g, b)
            }
            name if name.parse::<i32>().is_ok() => ansi_term::Color::Fixed(name.parse().unwrap()),
            _ => ansi_term::Color::Fixed(7),
        };
        let color_set = ColorSet {
            name: name.to_string(),
            color: color,
        };
        return color_set;
    }
}

#[derive(Serialize, Deserialize)]
struct ColorPatternMapTemplate {
    name: String,
    patterns: Vec<String>,
    #[serde(rename = "color")]
    color_set: ColorSet,
}

use serde::ser::SerializeStruct;
impl Serialize for ColorSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!("{}", self.name).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ColorSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let color_set = ColorSet::new(&s);
        Ok(color_set)
    }
}

impl Colorizer {
    pub fn load_config_file(&mut self, filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
        let f = std::fs::File::open(filepath)?;
        let mut color_pattern_map_data_list: Vec<ColorPatternMapTemplate> =
            serde_yaml::from_reader(f)?;
        self.color_pattern_map_data_list
            .append(&mut color_pattern_map_data_list);
        // for debug
        // let yaml_str = serde_yaml::to_string(&self.color_pattern_map_data_list).unwrap();
        // println!("Serialized yaml = {:?}", yaml_str);
        Ok(())
    }
    pub fn new() -> Self {
        let color_pattern_map_data_list: Vec<ColorPatternMapTemplate> = vec![
            // e.g.
            // ColorPatternMapTemplate {
            //     name: String::from("ip_addr"),
            //     patterns: vec![
            //         r"[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}",
            //     ]
            //     .into_iter()
            //     .map(String::from)
            //     .collect(),
            //     color_set: ColorSet::new("228"),
            // },
        ];

        let color_pattern_maps: Vec<ColorPatternMap> = vec![];
        let colorizer = Colorizer {
            color_pattern_map_data_list,
            color_pattern_maps,
        };
        return colorizer;
    }
    pub fn setup(&mut self) {
        self.color_pattern_map_data_list
            .push(ColorPatternMapTemplate {
                name: String::from("default"),
                patterns: vec![r".+"].into_iter().map(String::from).collect(),
                color_set: ColorSet::new("32"),
            });
        for color_pattern_map_data in self.color_pattern_map_data_list.iter() {
            let pattern = color_pattern_map_data.patterns.join("|");
            let m = ColorPatternMap::new(
                &color_pattern_map_data.name,
                &pattern,
                color_pattern_map_data.color_set.color,
            );
            self.color_pattern_maps.push(m);
        }
    }
    pub fn colorize(&self, s: &str) -> String {
        return process_color_pattern_maps(s, &self.color_pattern_maps);
    }
}
