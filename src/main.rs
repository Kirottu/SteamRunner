mod game_config;
mod ui;

use clap::{App, Arg};
use game_config::{ConfigOption, GameConfig};
use notify_rust::Notification;
use std::fs::File;
use std::process::{Child, Stdio};
use std::{
    env, fs,
    path::Path,
    process::{exit, Command},
    sync::{Arc, Mutex},
};

// The game and global config variables need to be defined like this to allow their usage in the ui
// code callbacks
lazy_static::lazy_static! {
    pub static ref GAME_CONFIG: Arc<Mutex<GameConfig>> = Arc::new(Mutex::new(GameConfig::default()));
    pub static ref GLOBAL_CONFIG: Arc<Mutex<GameConfig>> = Arc::new(Mutex::new(GameConfig::default()));
}

fn main() {
    let matches = App::new("SteamRunner")
        .arg(Arg::with_name("command").required(true).takes_value(true))
        .arg(Arg::with_name("nosteam")
                .help("Use specified identifier instead of steam appid (allows usage outside of steam)")
                .long("no-steam")
                .takes_value(true))
        .arg(Arg::with_name("banner")
             .help("Use this image as banner instead of trying to find one from steam cache using the appid.")
             .long("banner")
             .takes_value(true))
        .arg(Arg::with_name("logo")
             .help("Same as banner, but for the logo")
             .long("logo")
             .takes_value(true))
        .arg(Arg::with_name("nogui")
             .help("Disable the configuration gui")
             .long("no-gui")
             .takes_value(false))
        .arg(Arg::with_name("log")
             .help("Write game output to a log file")
             .long("log")
             .takes_value(false))
        .get_matches();

    let command = matches.value_of("command").unwrap();

    // Parse the initial command variable supplied by steam to find the steam AppId, which is used
    // for identifying the game specific config files
    let nosteam = matches.is_present("nosteam");

    let appid = if nosteam {
        matches.value_of("nosteam").unwrap().to_string()
    } else {
        let appid_index = match command.find("AppId=") {
            Some(appid_index) => appid_index,
            None => {
                println!(
                    "Failed to find appid from command string, did you mean to use --no-steam?"
                );
                exit(1);
            }
        };
        let mut appid_str: String = String::new();

        for chr in command.chars().skip(appid_index) {
            if chr == ' ' {
                break;
            }
            appid_str.push(chr);
        }
        appid_str.split("=").last().unwrap().to_string()
    };

    let banner_path = if matches.is_present("banner") {
        matches.value_of("banner").unwrap().to_string()
    } else {
        format!(
            "{}/.local/share/Steam/appcache/librarycache/{}_library_hero.jpg",
            env::var("HOME").unwrap(),
            appid,
        )
    };
    let logo_path = if matches.is_present("logo") {
        matches.value_of("logo").unwrap().to_string()
    } else {
        format!(
            "{}/.local/share/Steam/appcache/librarycache/{}_logo.png",
            env::var("HOME").unwrap(),
            appid
        )
    };

    // Get the config directory using environmental variables, and falling back to a standard path
    // if the environmental variables do not exist. $HOME is although required
    let config_dir = env::var("XDG_CONFIG_HOME")
        .unwrap_or(format!("{}/.config/steamrunner", env::var("HOME").unwrap()));

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
                appid: "".to_string(),
                placeholder_launch_command: "%mh% %ov% %og% %command%".to_string(),
                launch_command_modified: false,
                placeholder_map: vec![
                    ConfigOption::new(&"%mh%".to_string(), &"mangohud ".to_string(), false, false),
                    ConfigOption::new(
                        &"%ov%".to_string(),
                        &"obs-vkcapture ".to_string(),
                        false,
                        false,
                    ),
                    ConfigOption::new(
                        &"%og%".to_string(),
                        &"obs-glcapture ".to_string(),
                        false,
                        false,
                    ),
                ],
                pre_launch_commands: Vec::new(),
                post_exit_commands: Vec::new(),
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
            game_config.appid = appid.clone();
            create_new_game_config(&config_dir, &game_config, &game_config.appid)
        };
    if !matches.is_present("nogui") {
        if ui::run(&GLOBAL_CONFIG, &GAME_CONFIG, &banner_path, &logo_path) {
            exit(1);
        }
    }

    let game_config = GAME_CONFIG.lock().unwrap();

    // Run all pre launch commands and wait for them to exit before starting game.
    for command in &game_config.pre_launch_commands {
        if command.enabled {
            let mut handle = match Command::new("sh").arg("-c").arg(&command.command).spawn() {
                Ok(handle) => handle,
                Err(why) => {
                    Notification::new()
                        .summary("Failed to run pre-launch command")
                        .body(&format!(
                            "Failed to run command {}: {}",
                            &command.command, why
                        ))
                        .icon("notification_error")
                        .show()
                        .unwrap();
                    continue;
                }
            };
            handle.wait().unwrap();
        }
    }

    // Start the game and wait until it exits
    let handle: Option<Child> = match Command::new("sh")
        .arg("-c")
        .arg(game_config.get_launch_command(&command.to_string()))
        .stdout(if matches.is_present("log") {
            Stdio::from(
                File::create(format!("{}/logs/{}_stdout.log", &config_dir, &appid)).unwrap(),
            )
        } else {
            Stdio::null()
        })
        .stderr(if matches.is_present("log") {
            Stdio::from(
                File::create(format!("{}/logs/{}_stderr.log", &config_dir, &appid)).unwrap(),
            )
        } else {
            Stdio::null()
        })
        .spawn()
    {
        Ok(handle) => Some(handle),
        Err(why) => {
            Notification::new()
                .summary("Failed to run game")
                .body(&format!("Game startup failed: {}", why))
                .icon("notification_error")
                .show()
                .unwrap();
            None
        }
    };

    // If the game launched successfully, wait until it closes
    if let Some(mut handle) = handle {
        handle.wait().unwrap();
    }

    for command in &game_config.post_exit_commands {
        if command.enabled {
            let mut handle = match Command::new("sh").arg("-c").arg(&command.command).spawn() {
                Ok(handle) => handle,
                Err(why) => {
                    Notification::new()
                        .summary("Failed to run post-exit command")
                        .body(&format!(
                            "Failed to run command {}: {}",
                            &command.command, why
                        ))
                        .icon("notification_error")
                        .show()
                        .unwrap();
                    continue;
                }
            };
            handle.wait().unwrap();
        }
    }
}

fn create_config_dirs(config_dir: &String) {
    fs::create_dir(config_dir).unwrap_or(());
    fs::create_dir(&format!("{}/game_configs", config_dir)).unwrap_or(());
    fs::create_dir(&format!("{}/logs", config_dir)).unwrap_or(());
}

fn create_new_game_config(
    config_dir: &String,
    global_config: &GameConfig,
    appid: &String,
) -> GameConfig {
    if !Path::new(&format!("{}/game_configs", config_dir)).exists() {
        create_config_dirs(config_dir);
    }

    global_config.save(&format!("{}/game_configs/{}.yaml", config_dir, appid));

    (*global_config).clone()
}
