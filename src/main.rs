extern crate serde_json;

mod inotify;
mod config;
mod torrent;

use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use inotify::InotifyInstance;
use config::Config;

fn main() {	//todo: logging
	println!("Magnatar forming...");
	let mut config_file = match File::open("src/magnatar.json") {
		Ok(f) => f,
		Err(e) => {
			println!("Error while opening config file: {}", e);
			return;
		},
	};
	
	let mut json = String::with_capacity(1024);
	if let Err(e) = config_file.read_to_string(&mut json) {
		println!("Error while reading data: {}", e);
		return;
	}
	
	let config = match Config::from_json(&json) {
		Ok(c) => c,
		Err(e) => {
			println!("{}", e);
			return;
		},
	};
	
	let mut notify = match InotifyInstance::new() {
		Ok(n) => n,
		Err(e) => {
			println!("Error while creating inotify instance: {}", e);
			return;
		},
	};
	if let Err(e) = notify.add_watch(config.watch_path.as_path()) {
		println!("Error while adding watch: {}", e);
		return;
	}
	
	let mut buffer = Vec::<u8>::with_capacity(131072);	//128 KBs
	
	println!("Magnatar attracting...");
	notify.process_events(|filename: &Path| {
		let mut torrent_file = match File::open(config.watch_path.join(filename).as_path()) {
			Ok(f) => f,
			Err(e) => {
				println!("{}: failed to open file: {}", filename.to_string_lossy(), e);
				return;
			},
		};
		buffer.clear();
		if let Err(e) = torrent_file.read_to_end(&mut buffer) {
			println!("{}: error while reading: {}", filename.to_string_lossy(), e); //todo
		};
		match torrent::trackers(&buffer) {
			Ok(trackers) => {
				if let Some(dest) = config.destination(trackers) {
					println!("{}: sent on a trajectory to {}", filename.to_string_lossy(), dest.to_string_lossy());
					//and move the file
				}
			},
			Err(e) => println!("{}: {}", filename.to_string_lossy(), e),
		}
	});
}
