use regex::Regex;
use std::fs::File;
use std::io::prelude::*;

const APP_NAME: &'static str = "ProjectPeril";
const APP_VERSION_MAJOR: &'static str = env!("CARGO_PKG_VERSION_MAJOR");
const APP_VERSION_MINOR: &'static str = env!("CARGO_PKG_VERSION_MINOR");
const APP_VERSION_PATCH: &'static str = env!("CARGO_PKG_VERSION_PATCH");

pub struct Config {
    pub app_name: &'static str,
    pub app_version: u32,
    pub window_dimensions: (u32, u32),
    pub render_dimensions: (u32, u32),
}

impl Config {
    fn parse_option(&mut self, option: &str, value: &str) {
        match option {
            "window_width" => {
                let val: u32 = value.parse().expect("window width is NAN");
                self.window_dimensions.0 = val;
            }
            "window_height" => {
                let val: u32 = value.parse().expect("window height is NAN");
                self.window_dimensions.1 = val;
            }
            "render_width" => {
                let val: u32 = value.parse().expect("render width is NAN");
                self.render_dimensions.0 = val;
            }
            "render_height" => {
                let val: u32 = value.parse().expect("render height is NAN");
                self.render_dimensions.1 = val;
            }
            _ => println!("Invalid option: {} with value: {}", option, value),
        }

    }

    fn make_version(major: u32, minor: u32, patch: u32) -> u32 {
        debug_assert!(major <= 0x3FF); // 10 bit major
        debug_assert!(minor <= 0x3FF); // 10 bit minor
        debug_assert!(patch <= 0xFFF); // 12 bit patch

        let mut ret: u32 = 0;
        ret |= patch & 0xFFF;
        ret |= (minor & 0x3FF) << 12;
        ret |= (major & 0x3FF) << 24;

        ret
    }

    pub fn version_to_string(&self) -> String {
        let major = (self.app_version >> 24) & 0x3FF;
        let minor = (self.app_version >> 12) & 0x3FF;
        let patch = self.app_version & 0xFFF;

        format!("v{}.{}.{}", major, minor, patch)
    }

    pub fn read_config(filename: &str) -> Config {
        let mut file = File::open(filename).expect("Error opening config file");

        let mut contents = String::new();
        file.read_to_string(&mut contents).expect(
            "Error reading config file",
        );

        let mut cfg = Config {
            app_name: APP_NAME,
            app_version: Config::make_version(
                APP_VERSION_MAJOR.parse().unwrap(),
                APP_VERSION_MINOR.parse().unwrap(),
                APP_VERSION_PATCH.parse().unwrap(),
            ),
            window_dimensions: (0, 0),
            render_dimensions: (0, 0),
        };

        // match on option=value
        let re = Regex::new(r"(\w+)=(\w+)").unwrap();
        for rematch in re.captures_iter(&contents) {
            cfg.parse_option(&rematch[1], &rematch[2]);
        }

        cfg
    }
}
