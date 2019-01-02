#![allow(dead_code)]

extern crate clap;

use clap::{Arg, App};

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

struct Config {
	// Modes
	platform: TargetPlatform,
	filesync: FileSyncMode,

	// Flags
	is_patch: bool,

	// Derived
	destination: String,
}

fn parse_config() -> Config {
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

	// Destination
	let destination = "".to_string();

	return Config {platform, filesync, is_patch, destination};
}

fn main() {
	let config = parse_config();
	println!("{}", config.destination);
}
