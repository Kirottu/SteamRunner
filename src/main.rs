mod game_config;

use clap::{App, Arg};
use game_config::{ConfigOption, GameConfig};
use std::{env, fs, path::Path, process::Command};

fn main() {
    let matches = App::new("SteamTinkerLaunch-rs")
        .arg(Arg::with_name("command").required(true).takes_value(true))
        .get_matches();

    let command = matches.value_of("command").unwrap();
    let appid_index = command.find("AppId=").unwrap();
    let mut appid_str: String = String::new();

    for chr in command.chars().skip(appid_index) {
        if chr == ' ' {
            break;
        }
        appid_str.push(chr);
    }

    let appid = appid_str.split("=").last().unwrap();

    let config_dir = env::var("XDG_CONFIG_HOME")
        .unwrap_or(format!("{}/.config/stl-rs", env::var("HOME").unwrap()));

    if !Path::new(&config_dir).exists() {
        create_config_dirs(&config_dir);
    }

    let global_config = if Path::new(&format!("{}/global_config.yaml", config_dir)).exists() {
        GameConfig::load(&format!("{}/global_config.yaml", config_dir))
    } else {
        let global_config = GameConfig {
            placeholder_launch_command: "%mangohud%%obs-vkcapture%%obs-glcapture% %command%"
                .to_string(),
            placeholder_map: vec![
                ConfigOption::new(&"%mangohud%".to_string(), &"mangohud ".to_string(), false),
                ConfigOption::new(
                    &"%obs-vkcapture%".to_string(),
                    &"obs-vkcapture ".to_string(),
                    false,
                ),
                ConfigOption::new(
                    &"%obs-glcapture%".to_string(),
                    &"obs-glcapture ".to_string(),
                    false,
                ),
            ],
        };

        global_config.save(&format!("{}/global_config.yaml", config_dir));
        global_config
    };

    println!("{:?}", global_config);

    Command::new("sh")
        .arg("-c")
        .arg(global_config.get_launch_command(&command.to_string()))
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
