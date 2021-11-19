sixtyfps::include_modules!();

use std::{env, path::Path};

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

pub fn run(global_config: &mut GameConfig, game_config: &mut GameConfig, appid: u32) -> bool {
    let main_window = Main::new();

    // Load the banner image from steam library cache
    let banner_path = format!(
        "{}/.local/share/Steam/appcache/librarycache/{}_library_600x900.jpg",
        env::var("HOME").unwrap(),
        appid
    );
    let banner = sixtyfps::Image::load_from_path(Path::new(&banner_path)).unwrap();

    main_window.set_banner(banner);
    main_window.set_appid(sixtyfps::SharedString::from(format!("{}", appid)));

    // Convert the config options into something the UI code can understand and work on
    let game_config_options: Vec<sixtyfps_generated_Main::SixtyConfigOption> = game_config
        .placeholder_map
        .iter()
        .map(|option| (*option).clone().into())
        .collect();

    let global_config_options: Vec<sixtyfps_generated_Main::SixtyConfigOption> = global_config
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

    main_window.on_quit(move || {
        sixtyfps::quit_event_loop();
    });

    main_window.on_print_val(move |index: i32| {
        println!("{}", index);
    });

    main_window.run();

    main_window.get_launch_cancelled()
}
