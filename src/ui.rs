sixtyfps::include_modules!();

use std::{
    env,
    path::Path,
    sync::{Arc, Mutex},
};

use sixtyfps::SharedString;

use crate::{game_config::ConfigOption, GameConfig};

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

pub fn run(
    global_config: &'static Arc<Mutex<GameConfig>>,
    game_config: &'static Arc<Mutex<GameConfig>>,
) -> bool {
    let main_window = Main::new();

    // Load the banner image from steam library cache
    let banner_path = format!(
        "{}/.local/share/Steam/appcache/librarycache/{}_library_600x900.jpg",
        env::var("HOME").unwrap(),
        game_config.lock().unwrap().appid,
    );
    let banner = sixtyfps::Image::load_from_path(Path::new(&banner_path)).unwrap();

    main_window.set_banner(banner);
    main_window.set_appid(sixtyfps::SharedString::from(format!(
        "{}",
        game_config.lock().unwrap().appid
    )));

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
    let on_save_global_config_lock = Arc::clone(&global_config);
    let on_save_game_config_lock = Arc::clone(&game_config);
    main_window.on_save_config(move |is_game_config: bool| {
        if is_game_config {
            let game_config = on_save_game_config_lock.lock().unwrap();
            game_config.save(&format!(
                "{}/.config/stl-rs/game_configs/{}.yaml",
                env::var("HOME").unwrap(),
                game_config.appid
            ));
            println!("Saved game config");
        } else {
            let global_config = on_save_global_config_lock.lock().unwrap();
            global_config.save(&format!(
                "{}/.config/stl-rs/global_config.yaml",
                env::var("HOME").unwrap()
            ));
            println!("Saved global config");
        }
    });

    // Clones of the Arcs for the config structs.
    let on_sync_global_config_opt_lock = Arc::clone(&global_config);
    let on_sync_game_config_opt_lock = Arc::clone(&game_config);
    main_window.on_sync_config_opt(
        move |index: i32, config_opt: SixtyConfigOption, is_game_config: bool| {
            if is_game_config {
                let mut game_config = on_sync_game_config_opt_lock.lock().unwrap();
                game_config.placeholder_map[index as usize] = config_opt.clone().into();
                println!("{} {:?}", index, config_opt);
                println!(
                    "{} {:?}",
                    index, game_config.placeholder_map[index as usize]
                );
            } else {
                let mut global_config = on_sync_global_config_opt_lock.lock().unwrap();
                global_config.placeholder_map[index as usize] = config_opt.clone().into();
                println!("{} {:?}", index, config_opt);
                println!(
                    "{} {:?}",
                    index, global_config.placeholder_map[index as usize]
                );
            }
        },
    );

    let on_sync_placeholder_global_lock = Arc::clone(&global_config);
    let on_sync_placeholder_game_lock = Arc::clone(&game_config);
    main_window.on_sync_placeholder_string(move |text: SharedString, is_game_config: bool| {
        if is_game_config {
            on_sync_placeholder_game_lock
                .lock()
                .unwrap()
                .placeholder_launch_command = text.into();
            on_sync_placeholder_game_lock
                .lock()
                .unwrap()
                .launch_command_modified = true;
        } else {
            on_sync_placeholder_global_lock
                .lock()
                .unwrap()
                .placeholder_launch_command = text.into();
            on_sync_placeholder_global_lock
                .lock()
                .unwrap()
                .launch_command_modified = true;
        }
    });

    let on_add_item_main_window_weak = main_window.as_weak();
    let on_global_add_item_lock = Arc::clone(&global_config);
    let on_game_add_item_lock = Arc::clone(&game_config);
    main_window.on_add_item(move |is_game_config: bool| {
        let main_window = on_add_item_main_window_weak.unwrap();
        if is_game_config {
            let mut game_config = on_game_add_item_lock.lock().unwrap();
            game_config.placeholder_map.push(ConfigOption::new(
                &"".to_string(),
                &"".to_string(),
                false,
                false,
            ));
            game_config.set_on_sixtyfps(&main_window, true);
        } else {
            let mut global_config = on_global_add_item_lock.lock().unwrap();
            global_config.placeholder_map.push(ConfigOption::new(
                &"".to_string(),
                &"".to_string(),
                false,
                false,
            ));
            global_config.set_on_sixtyfps(&main_window, false);
        }
    });

    let on_merge_items_main_window_weak = main_window.as_weak();
    let on_merge_items_global_lock = Arc::clone(&global_config);
    let on_merge_items_game_lock = Arc::clone(&game_config);
    main_window.on_merge_global_into_game(move || {
        let mut game_config = on_merge_items_game_lock.lock().unwrap();
        let global_config = on_merge_items_global_lock.lock().unwrap();
        let main_window = on_merge_items_main_window_weak.unwrap();

        game_config.merge_with(&global_config);
        game_config.set_on_sixtyfps(&main_window, true);
    });

    main_window.run();

    main_window.get_launch_cancelled()
}
