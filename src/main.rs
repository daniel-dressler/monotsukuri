#![allow(dead_code)]

extern crate clap;
extern crate ini;
#[macro_use]
extern crate measure_time;

use ini::Ini;
use clap::{Arg, App};
use std::process::Command;
use std::fs;

enum TargetPlatform {
	Windows,
	PS4,
	Switch,
	XboxOne,
	MacOS,
	Linux,
}

#[derive(PartialEq)]
enum FileSyncMode {
	Fresh,
	Quick,
	Stale,
}

enum BuildMode {
	Dev,
	Release,
}


struct Config {
	// Project
	project_folder: String,
	project_file: String,
	scratch_folder: String,

	// Dropbox
	dropbox_folder: String,

	// SDKs
	nintendo_sdk_folder: String,
	steam_sdk_folder: String,

	// Unreal
	ue_src_folder: String,

}

struct Instructions {
	// Modes
	platform: TargetPlatform,
	filesync: FileSyncMode,
	buildmode: BuildMode,

	// Flags
	is_patch: bool,
	is_shipping: bool,

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
	let is_shipping = matches.value_of("ship") != None;

	// Build Mode
	let mut buildmode = BuildMode::Dev;
	if matches.value_of("release") != None {
		buildmode = BuildMode::Release;
	}

	// Destination
	let destination = "".to_string();

	return Instructions {platform, filesync, buildmode, is_patch, is_shipping, destination};
}


fn parse_config(_instructions : &Instructions) -> Config {

    let conf = Ini::load_from_file("monotsukuri.ini").unwrap();

    let section = conf.section(Some("paths")).unwrap();

	let dropbox_folder = section.get("dropbox").unwrap().to_string();
	let nintendo_sdk_folder = section.get("nintendo_sdk").unwrap().to_string();
	let steam_sdk_folder = section.get("steam_sdk").unwrap().to_string();
	let ue_src_folder = section.get("ue").unwrap().to_string();
	let project_folder = section.get("project").unwrap().to_string();
	let project_file = section.get("project_file").unwrap().to_string();
	let scratch_folder = section.get("scratch").unwrap().to_string();

	return Config {
		project_folder,
		project_file,
		scratch_folder,
		dropbox_folder,
		nintendo_sdk_folder,
		steam_sdk_folder,
		ue_src_folder,
	};
}

fn prepare_environment(instructions : &Instructions, config : &Config) {
	info_time!("Preparing Environment");

	// Set environmental variables

	// Sync Project into ScratchArea
	if instructions.filesync == FileSyncMode::Fresh {
		// Delete old sync
		fs::remove_dir_all(&config.scratch_folder).expect("Failed to delete scratch folder");
	}

	if instructions.filesync != FileSyncMode::Stale {
		// Decide Folders to Sync
		let mut sync_folders = vec![
			"Build".to_string(),
			"Config".to_string(),
			"Content".to_string(),
			"Source".to_string(),
			"Saved/Logs".to_string()
		];

		if instructions.is_patch {
			sync_folders.push("Releases".to_string());
		}

		for folder in &sync_folders {
			let mut src = config.project_folder.to_string();
			src.push_str("/");
			src.push_str(&folder);

			let mut dest = config.scratch_folder.to_string();
			dest.push_str("/");
			dest.push_str(&folder);

			Command::new("robocopy")
				.arg(src)
				.arg(dest)
				.arg("/MIR")
				.arg("/Z")
				.arg("/UNICODE")
				.arg("/NFL")
				.arg("/NDL")
		        .spawn()
				.expect("Failed to run Robocopy")
				.wait()
				.expect("Failed to finish Robocopy");
		}

		// Copy Files
		// Note: Robocopy expects to sync folders
		// it has weird syntax for a single file
		let sync_files = vec![
			config.project_file.to_string()
		];
		for file in &sync_files {
			let mut src = config.project_folder.to_string();
			src.push_str("/");
			src.push_str(&file);

			let mut dest = config.scratch_folder.to_string();
			dest.push_str("/");
			dest.push_str(&file);

			fs::copy(src, dest).expect("File copy failed");
		}
	}

}

fn compute_args(instructions : &Instructions, config : &Config) -> Vec<String> {

	// Find folder to run build from
	// Prepare should have moved project into this scratch
	let mut project = "-project=\"".to_string();
	project.push_str(&config.scratch_folder);
	project.push_str("/");
	project.push_str(&config.project_file);
	project.push_str("\"");

	// If Shipping mode
	let mut shipping = "".to_string();
	if instructions.is_shipping {
		shipping.push_str(" -clientconfig=Shipping -serverconfig=Shipping ");
	}

	let args = vec![
		"BuildCookRun".to_string(),
		project,
		"-platform=Win32".to_string(),
		"-compile".to_string(),
		"-cook".to_string(),
		"-build".to_string(),
		"-stage".to_string(),
		"-distribution".to_string(),
		"-SkipCookingEditorContent".to_string(),
		"-package".to_string(),
		shipping,
	];

	return args;
}

fn execute_build(instructions : &Instructions, config : &Config) {
	info_time!("Executing Build");

	let mut build_path = config.ue_src_folder.to_owned();
	build_path.push_str("/Engine/Build/BatchFiles/RunUAT.bat");

	println!("Calling: {}", build_path);

	// my $args = "-platform=$platform -cook -build -stage -distribution -SkipCookingEditorContent $compressArg -package ";
	// my $runUat = "$ue4/Engine/Build/BatchFiles/RunUAT.bat";
	// my $fullCall = "call $runUat BuildCookRun -project=\"$uproject\" $args $modeArgs $extraArgs";

	let args = compute_args(instructions, config);

	Command::new(build_path)
		.args(args)
        .spawn()
		.expect("Failed to RunUAT build")
		.wait()
		.expect("Build failed");
}

fn main() {
	let instructions = parse_instrctions();

	let config = parse_config(&instructions);

	prepare_environment(&instructions, &config);

	execute_build(&instructions, &config);

	println!("{}", instructions.destination);
}
