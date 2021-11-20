mod game_config;
mod ui;

use clap::{App, Arg};
use game_config::{ConfigOption, GameConfig};
use std::{
    env, fs,
    path::Path,
    process::{exit, Command},
    sync::{Arc, Mutex},
};

// The game and global config variables need to be defined like this to allow their usage in the ui
// code callbacks
lazy_static::lazy_static! {
    pub static ref GAME_CONFIG: Arc<Mutex<GameConfig>> = Arc::new(Mutex::new(
            GameConfig {
                appid: 0,
                placeholder_launch_command: "".to_string(),
                launch_command_modified: false,
                placeholder_map: Vec::new()
            }));
    pub static ref GLOBAL_CONFIG: Arc<Mutex<GameConfig>> = Arc::new(Mutex::new(
            GameConfig {
                appid: 0,
                placeholder_launch_command: "".to_string(),
                launch_command_modified: false,
                placeholder_map: Vec::new()
            }));
}

fn main() {
    let matches = App::new("SteamTinkerLaunch-rs")
        .arg(Arg::with_name("command").required(true).takes_value(true))
        .get_matches();

    let command = matches.value_of("command").unwrap();

    // Parse the initial command variable supplied by steam to find the steam AppId, which is used
    // for identifying the game specific config files
    let appid_index = command.find("AppId=").unwrap();
    let mut appid_str: String = String::new();

    for chr in command.chars().skip(appid_index) {
        if chr == ' ' {
            break;
        }
        appid_str.push(chr);
    }

    let appid = appid_str.split("=").last().unwrap().parse::<u32>().unwrap();

    // Get the config directory using environmental variables, and falling back to a standard path
    // if the environmental variables do not exist. $HOME is although required
    let config_dir = env::var("XDG_CONFIG_HOME")
        .unwrap_or(format!("{}/.config/stl-rs", env::var("HOME").unwrap()));

    // Make sure the config dir for stl-rs exists, if not create the directories
    if !Path::new(&config_dir).exists() {
        create_config_dirs(&config_dir);
    }

    // Load global config from the file if it exists, and fall back to a statically defined version
    // if the file does not exist
    *GLOBAL_CONFIG.lock().unwrap() =
        if Path::new(&format!("{}/global_config.yaml", config_dir)).exists() {
            GameConfig::load(&format!("{}/global_config.yaml", config_dir))
        } else {
            let global_config = GameConfig {
                appid: 0,
                placeholder_launch_command: "%mangohud% %obs-vkcapture% %obs-glcapture% %command%"
                    .to_string(),
                launch_command_modified: false,
                placeholder_map: vec![
                    ConfigOption::new(
                        &"%mangohud%".to_string(),
                        &"mangohud ".to_string(),
                        false,
                        false,
                    ),
                    ConfigOption::new(
                        &"%obs-vkcapture%".to_string(),
                        &"obs-vkcapture ".to_string(),
                        false,
                        false,
                    ),
                    ConfigOption::new(
                        &"%obs-glcapture%".to_string(),
                        &"obs-glcapture ".to_string(),
                        false,
                        false,
                    ),
                ],
            };

            // Save the newly created config file
            global_config.save(&format!("{}/global_config.yaml", config_dir));
            global_config
        };

    // Game specific config file
    // Load the file if it exists, otherwise create it and then return the global config
    *GAME_CONFIG.lock().unwrap() =
        if Path::new(&format!("{}/game_configs/{}.yaml", config_dir, appid)).exists() {
            GameConfig::load(&format!("{}/game_configs/{}.yaml", config_dir, appid))
        } else {
            let mut game_config = GLOBAL_CONFIG.lock().unwrap();
            game_config.appid = appid;
            create_new_game_config(&config_dir, &game_config, appid)
        };

    if ui::run(&GLOBAL_CONFIG, &GAME_CONFIG) {
        exit(1);
    }

    println!(
        "{}",
        GAME_CONFIG
            .lock()
            .unwrap()
            .get_launch_command(&command.to_string())
    );

    Command::new("sh")
        .arg("-c")
        .arg(
            GAME_CONFIG
                .lock()
                .unwrap()
                .get_launch_command(&command.to_string()),
        )
        .spawn()
        .unwrap();
}

fn create_config_dirs(config_dir: &String) {
    fs::create_dir(config_dir).unwrap_or(());
    fs::create_dir(&format!("{}/game_configs", config_dir)).unwrap();
}

fn create_new_game_config(
    config_dir: &String,
    global_config: &GameConfig,
    appid: u32,
) -> GameConfig {
    if !Path::new(&format!("{}/game_configs", config_dir)).exists() {
        create_config_dirs(config_dir);
    }

    global_config.save(&format!("{}/game_configs/{}.yaml", config_dir, appid));

    (*global_config).clone()
}
