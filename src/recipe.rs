use crate::cli::Arguments;

use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, fs};

use std::io::Read;

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize)]
pub enum _Recipe {
    interpolation {
        enabled: bool,
        fps: i32,
        speed: String,
        tuning: String,
        algorithm: i32,
        use_gpu: bool,
    },

    frame_blending {
        enabled: bool,
        fps: i32,
        intensity: f64,
        weighting: String,
    },

    flowblur {
        enabled: bool,
        amount: i32,
        mask: Option<PathBuf>,
    },

    encoding {
        process: PathBuf,
        loglevel: String,
        args: String,
    },

    preview {
        enabled: bool,
        ffmpeg: String,
        process: PathBuf,
        args: String,
    },

    misc {
        mpv_bin: PathBuf,
        stay_on_top: bool,

        ding_after: i32,
        folder: Option<PathBuf>,
        container: String,
        file_format: String,
        debug: bool,
        dedupthreshold: i32,
    },

    console_params {
        ontop: bool,
        borderless: bool,
        width: i32,
        height: i32,
    },

    timescale {
        input: f64,
        output: f64,
    },

    pre_interp {
        enabled: bool,
        factor: i32,
        model: PathBuf,
    },
}

fn parse_recipe(content: String) {

    let mut rc: HashMap<String, HashMap<&str, &str>> = HashMap::new();

    let mut round = 0;
    let mut cur_section = String::new();
    let lines: Vec<&str> = content.split("\n").collect();

    for line in lines {
        round += 1;
        let cur = line.trim();

        match cur {
            _comment 
                if cur.starts_with('#')  ||
                   cur.starts_with('/')  ||
                   cur.starts_with(';')  ||
                   cur.starts_with(':')  => {}

            _emptyline if cur.is_empty() => {}

            // e.g [frame interpolation]
            category if cur.starts_with('[') && cur.ends_with(']') => {
                cur_section = category
                    .trim_matches(|c| c == '[' || c == ']')
                    // remove all [ and ] characters
                    .trim()
                    // remove any spaces that would be at the start and end
                    .to_string();

                if !rc.contains_key(&cur_section) {
                    rc.insert(cur_section.clone(), HashMap::new());
                }
            }
            // e.g weighting: gaussian
            setting if cur.contains(':') => {

                if cur_section == String::new(){
                    panic!(
                        "Recipe: Setting {:?} has no parent category, line {round}",
                        setting
                    );
                }

                let (key, value) = setting.split_once(':')
                    .expect("Recipe: Failed to split {setting}, line {round}");

                rc.get_mut(&cur_section).unwrap().insert(key.trim(), value.trim());
            }
                            
            _ => panic!("Recipe: Failed to parse {:?}, line {round}", cur),
        }
    }
    println!("{:?}", rc);
}

pub fn get_recipe(args: &Arguments) {

    let exe = match env::current_exe() {
        Ok(exe) => exe,
        Err(e) => panic!("Could not resolve Smoothie's binary path: {}", e),
    };

    let bindir = match exe.parent() {
        Some(bindir) => bindir,
        None => panic!("Could not resolve Smoothie's binary directory `{:?}`", exe),
    };

    let rc_path = bindir.join(args.recipe.clone());

    assert!(
        rc_path.exists(),
        "Recipe at path `{:?}` does not exist",
        rc_path
    );

    println!("recipe: {:?}", rc_path);

    // Open the file at the given path
    let mut file = match fs::File::open(&rc_path) {
        Ok(file) => file,
        Err(e) => panic!("Error opening file: {}", e),
    };

    // Check if the file is empty
    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(e) => panic!("Error getting file metadata: {}", e),
    };
    if metadata.len() == 0 {
        panic!("Error: File is empty");
    }

    // Read the contents of the file into a string
    let mut content = String::new();
    match file.read_to_string(&mut content) {
        Ok(_) => (),
        Err(e) => panic!("Error reading file: {}", e),
    };

    parse_recipe(content);
}
