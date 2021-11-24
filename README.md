# SteamRunner
A tool for adding environmental variables, wrapper commands, pre-launch and post-exit commands and more to come.

# Usage
Clone the git repository and run `cargo install --path .` in the project directory. 
That command will build the program and install it into `~/.cargo/bin/` so make sure that it is included in `$PATH`.
After it is installed and in your `$PATH` (can be checked via `steamrunner --help`) you can use it by adding `steamrunner "%command%"` to the games launch options in steam.

# Configuration
The configuration files are stored in `~/.config/steamrunner` as follows:

`global_config.yaml`: All new game configs will be created based on this and changes in this can be easily merged to all other game confgis.
`game_configs/<appid>.yaml`: Game specific configs identified with their appid.

# Debugging
The argument `--log` can be used to enable redirecting the `stdout` and `stderr` of the game process into log files `logs/<appid>_stdout.log` and `logs/<appid>_stderr.log` in the configuration directory.

# Why
I wanted to create my own tool using a compiled language for fast and responsive operation.
I also wanted the tool to be extendable in the tools and utilities that it can use.

# Disclaimer
This is still very WIP and much of the functionality is subject to change. The codebase is also littered with many quick fixes which can cause panics.
