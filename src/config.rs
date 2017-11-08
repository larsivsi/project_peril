use regex::Regex;
use std::fs::File;
use std::io::prelude::*;

pub fn read_config() {
    let filename = "options.cfg";
    let mut file = File::open(filename).expect("Error opening config file");

    let mut contents = String::new();
    file.read_to_string(&mut contents).expect(
        "Error reading config file",
    );

    // match on option=value
    let re = Regex::new(r"(\w+)=(\w+)").unwrap();
    for rematch in re.captures_iter(&contents) {
        println!("captured: {} which is set to {}", &rematch[1], &rematch[2]);
    }
}
