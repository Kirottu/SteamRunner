sixtyfps::include_modules!();

use std::{
    env,
    path::Path,
    sync::{Arc, Mutex},
};

use crate::{
    game_config::{ConfigCommand, ConfigOption},
    GameConfig,
};
use closure::closure;
use sixtyfps::SharedString;

impl From<ConfigOption> for sixtyfps_generated_Main::SixtyConfigOption {
    fn from(config: ConfigOption) -> Self {
        Self {
            placeholder: SharedString::from(&config.placeholder),
            replace_with: SharedString::from(&config.replace_with),
            enabled: config.enabled,
            modified: config.modified,
        }
    }
}

impl From<SixtyConfigOption> for ConfigOption {
    fn from(config: SixtyConfigOption) -> Self {
        Self {
            placeholder: config.placeholder.into(),
            replace_with: config.replace_with.into(),
            enabled: config.enabled,
            modified: config.modified,
        }
    }
}

impl From<ConfigCommand> for sixtyfps_generated_Main::SixtyConfigCommand {
    fn from(command: ConfigCommand) -> Self {
        Self {
            command: SharedString::from(command.command),
            enabled: command.enabled,
            modified: command.modified,
        }
    }
}

impl From<SixtyConfigCommand> for ConfigCommand {
    fn from(command: SixtyConfigCommand) -> Self {
        Self {
            command: command.command.into(),
            enabled: command.enabled,
            modified: command.modified,
        }
    }
}

pub fn run(
    global_config: &'static Arc<Mutex<GameConfig>>,
    game_config: &'static Arc<Mutex<GameConfig>>,
    banner_path: &String,
    logo_path: &String,
) -> bool {
    let main_window = Main::new();
    let main_window_weak = main_window.as_weak();
    // Load the banner image from steam library cache
    match sixtyfps::Image::load_from_path(Path::new(banner_path)) {
        Ok(banner) => main_window.set_banner(banner),
        Err(why) => println!("Error loading banner: {:?}", why),
    }
    match sixtyfps::Image::load_from_path(Path::new(logo_path)) {
        Ok(logo) => main_window.set_logo(logo),
        Err(why) => println!("Error loading logo: {:?}", why),
    }

    main_window.set_game_launch_placeholder(sixtyfps::SharedString::from(
        game_config
            .lock()
            .unwrap()
            .placeholder_launch_command
            .clone(),
    ));
    main_window.set_global_launch_placeholder(sixtyfps::SharedString::from(
        global_config
            .lock()
            .unwrap()
            .placeholder_launch_command
            .clone(),
    ));

    // Convert the config options into something the UI code can understand and work on
    let game_config_options: Vec<sixtyfps_generated_Main::SixtyConfigOption> = game_config
        .lock()
        .unwrap()
        .placeholder_map
        .iter()
        .map(|option| (*option).clone().into())
        .collect();

    let global_config_options: Vec<sixtyfps_generated_Main::SixtyConfigOption> = global_config
        .lock()
        .unwrap()
        .placeholder_map
        .iter()
        .map(|option| (*option).clone().into())
        .collect();

    let game_pre_launch_commands: Vec<sixtyfps_generated_Main::SixtyConfigCommand> = game_config
        .lock()
        .unwrap()
        .pre_launch_commands
        .iter()
        .map(|command| (*command).clone().into())
        .collect();

    let game_post_exit_commands: Vec<sixtyfps_generated_Main::SixtyConfigCommand> = game_config
        .lock()
        .unwrap()
        .post_exit_commands
        .iter()
        .map(|command| (*command).clone().into())
        .collect();

    let global_pre_launch_commands: Vec<sixtyfps_generated_Main::SixtyConfigCommand> =
        global_config
            .lock()
            .unwrap()
            .pre_launch_commands
            .iter()
            .map(|command| (*command).clone().into())
            .collect();

    let global_post_exit_commands: Vec<sixtyfps_generated_Main::SixtyConfigCommand> = global_config
        .lock()
        .unwrap()
        .post_exit_commands
        .iter()
        .map(|command| (*command).clone().into())
        .collect();

    main_window.set_global_pre_launch_commands(sixtyfps::ModelHandle::new(std::rc::Rc::new(
        sixtyfps::VecModel::from(global_pre_launch_commands),
    )));
    main_window.set_global_post_exit_commands(sixtyfps::ModelHandle::new(std::rc::Rc::new(
        sixtyfps::VecModel::from(global_post_exit_commands),
    )));

    main_window.set_game_pre_launch_commands(sixtyfps::ModelHandle::new(std::rc::Rc::new(
        sixtyfps::VecModel::from(game_pre_launch_commands),
    )));
    main_window.set_game_post_exit_commands(sixtyfps::ModelHandle::new(std::rc::Rc::new(
        sixtyfps::VecModel::from(game_post_exit_commands),
    )));

    main_window.set_game_config_options(sixtyfps::ModelHandle::new(std::rc::Rc::new(
        sixtyfps::VecModel::from(game_config_options),
    )));
    main_window.set_global_config_options(sixtyfps::ModelHandle::new(std::rc::Rc::new(
        sixtyfps::VecModel::from(global_config_options),
    )));

    // Callbacks from the ui code are defined here

    main_window.on_quit(move || {
        sixtyfps::quit_event_loop();
    });

    main_window.on_print_val(move |index: i32| {
        println!("{}", index);
    });

    // Clones of the Arcs for the config structs.
    main_window.on_save_config(move |is_game_config: bool| {
        let (config, save_path) = if is_game_config {
            let game_config = game_config.lock().unwrap();
            let appid = game_config.appid.clone();
            (
                game_config,
                format!(
                    "{}/.config/stl-rs/game_configs/{}.yaml",
                    env::var("HOME").unwrap(),
                    appid
                ),
            )
        } else {
            (
                global_config.lock().unwrap(),
                format!(
                    "{}/.config/stl-rs/global_config.yaml",
                    env::var("HOME").unwrap()
                ),
            )
        };
        config.save(&save_path);
    });

    // Clones of the Arcs for the config structs.
    main_window.on_sync_config_opt(
        move |index: i32, config_opt: SixtyConfigOption, is_game_config: bool| {
            let mut config = if is_game_config {
                game_config.lock().unwrap()
            } else {
                global_config.lock().unwrap()
            };
            config.placeholder_map[index as usize] = config_opt.clone().into();
        },
    );

    main_window.on_sync_config_cmd(
        closure!(clone game_config, clone global_config, | index: i32,
        command: SixtyConfigCommand,
        is_pre_launch: bool, 
        is_game_config: bool | {
            let mut config = if is_game_config { game_config.lock().unwrap() } else { global_config.lock().unwrap() };
            if is_pre_launch {
                config.pre_launch_commands[index as usize] = command.into();
            } else {
                config.post_exit_commands[index as usize] = command.into();
            }
        }),
    );

    main_window.on_sync_placeholder_string(closure!(clone game_config, clone global_config, |text: SharedString, is_game_config: bool| {
        let mut config = if is_game_config { game_config.lock().unwrap() } else { global_config.lock().unwrap() };
        config.placeholder_launch_command = text.into();
        config.launch_command_modified = true;
    }));

    main_window.on_add_config_opt(closure!(clone main_window_weak, clone game_config, clone global_config, |is_game_config: bool| {
        let main_window = main_window_weak.unwrap();
        let mut config = if is_game_config { game_config.lock().unwrap() } else { global_config.lock().unwrap() };
        config.placeholder_map.push(ConfigOption::default());
        config.set_on_sixtyfps(&main_window, is_game_config);
    }));

    main_window.on_add_config_cmd(closure!(clone main_window_weak, clone game_config, clone global_config, |is_pre_launch: bool, is_game_config: bool| {
        let main_window = main_window_weak.unwrap();
        let mut config = if is_game_config { game_config.lock().unwrap() } else { global_config.lock().unwrap() };
        if is_pre_launch {
            config.pre_launch_commands.push(ConfigCommand::default());
        } else {
            config.post_exit_commands.push(ConfigCommand::default());
        }
        config.set_on_sixtyfps(&main_window, is_game_config);
    }));

    main_window.on_remove_config_opt(closure!(clone main_window_weak, clone game_config, clone global_config, |index: i32, is_game_config: bool| {
        let main_window = main_window_weak.unwrap();
        let mut config = if is_game_config { game_config.lock().unwrap() } else { global_config.lock().unwrap() };
        config.placeholder_map.remove(index as usize);
        config.set_on_sixtyfps(&main_window, is_game_config);
    }));
    main_window.on_remove_config_cmd(closure!(clone main_window_weak, clone game_config, clone global_config, |index: i32, is_pre_launch: bool, is_game_config: bool| {
        let main_window = main_window_weak.unwrap();
        let mut config = if is_game_config { game_config.lock().unwrap() } else { global_config.lock().unwrap() };
        if is_pre_launch {
            config.pre_launch_commands.remove(index as usize);
        } else {
            config.post_exit_commands.remove(index as usize);
        }
        config.set_on_sixtyfps(&main_window, is_game_config);
    }));

    main_window.on_merge_global_into_game(
        closure!(clone main_window_weak, clone game_config, clone global_config, || {
            let mut game_config = game_config.lock().unwrap();
            let global_config = global_config.lock().unwrap();
            let main_window = main_window_weak.unwrap();

            game_config.merge_with(&global_config);
            game_config.set_on_sixtyfps(&main_window, true);
        }),
    );

    main_window.run();

    main_window.get_launch_cancelled()
}
