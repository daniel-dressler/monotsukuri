#![allow(dead_code)]

extern crate clap;
extern crate ini;

use ini::Ini;
use clap::{Arg, App};
use std::process::{Command, Stdio};

enum TargetPlatform {
	Windows,
	PS4,
	Switch,
	XboxOne,
	MacOS,
	Linux,
}

enum FileSyncMode {
	Fresh,
	Quick,
	Stale,
}

enum BuildMode {
	Dev,
	Release,
}



struct Instructions {
	// Modes
	platform: TargetPlatform,
	filesync: FileSyncMode,
	buildmode: BuildMode,

	// Flags
	is_patch: bool,

	// Derived
	destination: String,
}

fn parse_instrctions() -> Instructions {
    let matches = App::new("Monotsukuri")
        .version("0.1.0")
        .author("Daniel Dressler <danieru.dressler@gmail.com")
        .about("Commercial grade meta builder for UE4 game product packaging")
        .arg(Arg::with_name("PLATFORM")
             .required(true)
             .takes_value(true)
             .index(1)
             .help("Platform to build packages for"))
		.arg(Arg::with_name("patch")
			.help("Create patch files"))
		.arg(Arg::with_name("quick")
			.help("Perform a Quick file sync"))
		.arg(Arg::with_name("nosync")
			.help("Do not file sync"))
		.arg(Arg::with_name("fresh")
			.help("Perform a full & fresh file sync"))
        .get_matches();

	// Platform
    let platform_raw = matches.value_of("PLATFORM").unwrap().to_lowercase();
	let platform = match platform_raw.as_ref() {
		"windows" => TargetPlatform::Windows,
		_ => TargetPlatform::Windows,
	};

	// File Sync Mode
	let mut filesync = FileSyncMode::Quick;
	if matches.value_of("nosync") != None {
		filesync = FileSyncMode::Stale;
	}
	if matches.value_of("fresh") != None {
		filesync = FileSyncMode::Fresh;
	}

	// Patch
	let is_patch = matches.value_of("patch") != None;

	// Build Mode
	let mut buildmode = BuildMode::Dev;
	if matches.value_of("release") != None {
		buildmode = BuildMode::Release;
	}

	// Destination
	let destination = "".to_string();

	return Instructions {platform, filesync, buildmode, is_patch, destination};
}

struct Config {
	// Dropbox
	dropbox_folder: String,

	// SDKs
	nintendo_sdk_folder: String,
	steam_sdk_folder: String,

	// Unreal
	ue_src_folder: String,

}

fn parse_config(instructions : &Instructions) -> Config {
	let dropbox_folder = "".to_string();
	let nintendo_sdk_folder = "".to_string();
	let steam_sdk_folder = "".to_string();
	let ue_src_folder = "".to_string();

    let conf = Ini::load_from_file("conf.ini").unwrap();

    let section = conf.section(Some("User".to_owned())).unwrap();
    let tommy = section.get("given_name").unwrap();
    let green = section.get("family_name").unwrap();

	return Config { dropbox_folder, nintendo_sdk_folder, steam_sdk_folder, ue_src_folder };
}

fn prepare_environment(config : &Config) {

	// Set environmental variables

}

fn execute_build(instructions : &Instructions, config : &Config) {
	let runUatPath = config.ue_src_folder + "/Engine/Build/BatchFiles/RunUAT.bat";
	Command::new(runUatPath)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .spawn();
}

fn main() {
	let instructions = parse_instrctions();

	let config = parse_config(&instructions);

	prepare_environment(&config);

	execute_build(&instructions, &config);

	println!("{}", instructions.destination);
}
