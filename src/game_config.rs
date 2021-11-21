use std::{fs::File, io::Write};

use serde::{Deserialize, Serialize};

// Struct to contain the values for one specific option
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ConfigOption {
    pub placeholder: String,
    pub replace_with: String,
    pub enabled: bool,
    pub modified: bool,
}

impl ConfigOption {
    pub fn new(placeholder: &String, replace_with: &String, enabled: bool, modified: bool) -> Self {
        Self {
            placeholder: placeholder.clone(),
            replace_with: replace_with.clone(),
            enabled: enabled.clone(),
            modified: modified.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ConfigCommand {
    pub command: String,
    pub enabled: bool,
    pub modified: bool,
}

// Entire struct to contain all ConfigOptions for the game specific config or the global config
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GameConfig {
    pub appid: u32,
    pub placeholder_launch_command: String,
    pub launch_command_modified: bool,
    pub placeholder_map: Vec<ConfigOption>,
    pub pre_launch_commands: Vec<ConfigCommand>,
    pub post_exit_commands: Vec<ConfigCommand>,
}

impl GameConfig {
    // Function for creating the command to launch the game with this specified config
    pub fn get_launch_command(&self, command: &String) -> String {
        let mut launch_command = self.placeholder_launch_command.clone();

        // Substitute the real values for the placeholders in the launch_command string
        for item in &self.placeholder_map {
            if item.enabled {
                launch_command = launch_command.replace(&item.placeholder, &item.replace_with);
            } else {
                launch_command = launch_command.replace(&item.placeholder, "");
            }
        }
        launch_command = launch_command.replace("%command%", &command); // Add the actual command supplied by steam

        launch_command
    }
    // Load the config file from the specified path
    pub fn load(path: &String) -> Self {
        let file = File::open(path).expect("Could not load file");

        serde_yaml::from_reader(file).unwrap()
    }
    // Save the config to a file in the specified path
    pub fn save(&self, path: &String) {
        let mut file = File::create(path).unwrap();
        file.write_all(&serde_yaml::to_string(self).unwrap().as_bytes())
            .unwrap();
    }

    pub fn merge_with(&mut self, config: &GameConfig) {
        for config_opt in &config.placeholder_map {
            let mut replaced = false;
            for self_opt in &mut self.placeholder_map {
                if !self_opt.modified && self_opt.placeholder == config_opt.placeholder {
                    self_opt.placeholder = config_opt.placeholder.clone();
                    self_opt.replace_with = config_opt.replace_with.clone();
                    replaced = true;
                    break;
                }
            }
            if !replaced {
                self.placeholder_map.push((*config_opt).clone());
            }
        }
        if !self.launch_command_modified {
            self.placeholder_launch_command = config.placeholder_launch_command.clone();
        }
    }

    pub fn set_on_sixtyfps(&self, main_window: &crate::ui::Main, is_game_config: bool) {
        let config_options: Vec<crate::ui::SixtyConfigOption> = self
            .placeholder_map
            .iter()
            .map(|option| (*option).clone().into())
            .collect();

        let pre_launch_commands: Vec<crate::ui::SixtyConfigCommand> = self
            .pre_launch_commands
            .iter()
            .map(|command| (*command).clone().into())
            .collect();

        let post_exit_commands: Vec<crate::ui::SixtyConfigCommand> = self
            .post_exit_commands
            .iter()
            .map(|command| (*command).clone().into())
            .collect();

        if is_game_config {
            main_window.set_game_config_options(sixtyfps::ModelHandle::new(std::rc::Rc::new(
                sixtyfps::VecModel::from(config_options),
            )));
            main_window.set_game_pre_launch_commands(sixtyfps::ModelHandle::new(std::rc::Rc::new(
                sixtyfps::VecModel::from(pre_launch_commands),
            )));
            main_window.set_game_post_exit_commands(sixtyfps::ModelHandle::new(std::rc::Rc::new(
                sixtyfps::VecModel::from(post_exit_commands),
            )));
        } else {
            main_window.set_global_config_options(sixtyfps::ModelHandle::new(std::rc::Rc::new(
                sixtyfps::VecModel::from(config_options),
            )));
            main_window.set_global_pre_launch_commands(sixtyfps::ModelHandle::new(
                std::rc::Rc::new(sixtyfps::VecModel::from(pre_launch_commands)),
            ));
            main_window.set_global_post_exit_commands(sixtyfps::ModelHandle::new(
                std::rc::Rc::new(sixtyfps::VecModel::from(post_exit_commands)),
            ));
        }
    }
}
