use serde_json;
use std::fs::File;
use std::io::{Error, ErrorKind};

const APP_NAME: &'static str = "ProjectPeril";
const APP_VERSION_MAJOR: &'static str = env!("CARGO_PKG_VERSION_MAJOR");
const APP_VERSION_MINOR: &'static str = env!("CARGO_PKG_VERSION_MINOR");
const APP_VERSION_PATCH: &'static str = env!("CARGO_PKG_VERSION_PATCH");

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub app_name: String,
    pub app_version: u32,
    pub horizontal_fov: u32,
    pub render_width: u32,
    pub render_height: u32,
    pub window_width: u32,
    pub window_height: u32,
}

impl Config {
    /// Generates a packed 32 bit version number based on the given major, minor and patch
    /// versions.
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

    /// Prints the current app version as a string.
    pub fn version_to_string(&self) -> String {
        let major = (self.app_version >> 24) & 0x3FF;
        let minor = (self.app_version >> 12) & 0x3FF;
        let patch = self.app_version & 0xFFF;

        format!("v{}.{}.{}", major, minor, patch)
    }

    /// Saves the Config to the supplied filename.
    fn save(&self, filename: &str) -> Result<(), Error> {
        let file = File::create(filename)?;
        match serde_json::to_writer_pretty(file, self) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    /// Either reads the config given by the filename and generates a Config struct,
    /// or creates a default config and saves it to disk if the config file is not found.
    pub fn read_config(filename: &str) -> Result<Config, Error> {
        let correct_name = String::from(APP_NAME);
        let correct_version = Config::make_version(
            APP_VERSION_MAJOR.parse().unwrap(),
            APP_VERSION_MINOR.parse().unwrap(),
            APP_VERSION_PATCH.parse().unwrap(),
        );

        match File::open(filename) {
            Ok(file) => {
                let mut cfg: Config = serde_json::from_reader(file)?;

                let mut needs_save = false;
                if cfg.app_name != correct_name {
                    cfg.app_name = correct_name;
                    needs_save = true;
                }
                if cfg.app_version != correct_version {
                    cfg.app_version = correct_version;
                    needs_save = true;
                }
                if needs_save {
                    cfg.save(filename)?;
                }

                Ok(cfg)
            }
            Err(e) => {
                match e.kind() {
                    ErrorKind::NotFound => {
                        println!("WARNING: Options file ({}) not found, creating new with default values.", filename);
                        let cfg = Config {
                            app_name: correct_name,
                            app_version: correct_version,
                            horizontal_fov: 90,
                            render_width: 480,
                            render_height: 320,
                            window_width: 480,
                            window_height: 320,
                        };
                        cfg.save(filename)?;
                        Ok(cfg)
                    }
                    _ => Err(e),
                }
            }
        }
    }
}
