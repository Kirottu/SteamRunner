use std::{fs::File, io::Write, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigOption {
    placeholder: String,
    replace_with: String,
    enabled: bool,
}

impl ConfigOption {
    pub fn new(placeholder: &String, replace_with: &String, enabled: bool) -> Self {
        Self {
            placeholder: placeholder.clone(),
            replace_with: replace_with.clone(),
            enabled: enabled.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameConfig {
    pub appid: u32,
    pub placeholder_launch_command: String,
    pub placeholder_map: Vec<ConfigOption>,
}

impl GameConfig {
    pub fn get_launch_command(&self, command: &String) -> String {
        let mut launch_command = self.placeholder_launch_command.clone();

        for item in &self.placeholder_map {
            if item.enabled {
                launch_command = launch_command.replace(&item.placeholder, &item.replace_with);
            } else {
                launch_command = launch_command.replace(&item.placeholder, "");
            }
        }
        launch_command = launch_command.replace("%command%", &command);

        launch_command
    }
    pub fn load(path: &String) -> Self {
        let file = File::open(path).expect("Could not load file");

        serde_yaml::from_reader(file).unwrap()
    }
    pub fn save(&self, path: &String) {
        let mut file = File::create(path).unwrap();
        file.write_all(&serde_yaml::to_string(self).unwrap().as_bytes())
            .unwrap();
    }
}
