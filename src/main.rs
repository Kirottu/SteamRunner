mod game_config;
mod ui;

use clap::{App, Arg};
use game_config::{ConfigOption, GameConfig};
use std::{
    env, fs,
    path::Path,
    process::{exit, Command},
};

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
    let mut global_config = if Path::new(&format!("{}/global_config.yaml", config_dir)).exists() {
        GameConfig::load(&format!("{}/global_config.yaml", config_dir))
    } else {
        let global_config = GameConfig {
            placeholder_launch_command: "%mangohud%%obs-vkcapture%%obs-glcapture% %command%"
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
    let mut game_config =
        if Path::new(&format!("{}/game_configs/{}.yaml", config_dir, appid)).exists() {
            GameConfig::load(&format!("{}/game_configs/{}.yaml", config_dir, appid))
        } else {
            create_new_game_config(&config_dir, &global_config, appid)
        };

    if ui::run(&mut global_config, &mut game_config, appid) {
        exit(1);
    }

    /*Command::new("sh")
    .arg("-c")
    .arg(game_config.get_launch_command(&command.to_string()))
    .spawn()
    .unwrap();*/
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
