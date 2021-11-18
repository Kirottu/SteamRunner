use std::{fs::File, io::Write};

use serde::{Deserialize, Serialize};

// Struct to contain the values for one specific option
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigOption {
    placeholder: String,
    replace_with: String,
    enabled: bool,
    modified: bool,
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

// Entire struct to contain all ConfigOptions for the game specific config or the global config
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameConfig {
    pub placeholder_launch_command: String,
    pub placeholder_map: Vec<ConfigOption>,
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
}
