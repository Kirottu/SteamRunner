use clap::{App, Arg};
use std::process::Command;

const VKCAPTURE_PLACEHOLDER: &str = "%obs% %command%";

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

    let mut launch_command = VKCAPTURE_PLACEHOLDER.to_string();

    if VKCAPTURE_PLACEHOLDER.contains("%obs%") {
        launch_command = launch_command.replace("%obs%", "obs-vkcapture");
    }
    launch_command = launch_command.replace("%command%", command);

    println!("\"{}\"", launch_command);

    Command::new("sh")
        .arg("-c")
        .arg(format!("'{}'", launch_command))
        .spawn()
        .unwrap();
}
