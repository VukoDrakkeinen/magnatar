extern crate serde;
extern crate serde_json;
extern crate aho_corasick;

use std::path::{Path, PathBuf};
use std::env;
use std::mem;
use std::ops::Deref;
use self::aho_corasick::{Automaton, AcAutomaton};

include!(concat!(env!("OUT_DIR"), "/config.rs")); 

trait ExpandTilde {
	fn expand_tilde(&mut self) -> &Path
	where Self: AsRef<Path>;
}

impl ExpandTilde for PathBuf {
	fn expand_tilde(&mut self) -> &Path {
		if let Ok(expanded) = self.strip_prefix("~").map_err(|_| ()).and_then(|latter_part| env::home_dir().ok_or(()).map(|home| home.join(latter_part))) {	//todo: how the fuck do I make this readable
			*self = expanded;
		};
		self.as_path()
	}
}

impl Config {
	pub fn from_json(json: &String) -> Result<Config, String> {
		match serde_json::from_str::<Config>(json) {
			Ok(mut config) => {
				config.watch_path.expand_tilde();
				for rule in config.rules.iter_mut() {
					rule.destination.expand_tilde();
				}
				let tracker_matcher = AcAutomaton::new(config.rules.iter_mut().map(|rule| &mut rule.pattern).filter_map(|ref mut pattern| mem::replace(&mut pattern.tracker, None)));
				if !tracker_matcher.is_empty() {
					config.matcher.of_trackers = Some(tracker_matcher);
				}
				Ok(config)
			},
			Err(e) => Err(format!("Error while decoding JSON data: {}", e)),
		}
	}
	
	//todo: also decide using other torrent properties
	pub fn destination<T, S>(&self, trackers: T) -> Option<&Path>	//todo: what about empty matches?
	where T: IntoIterator<Item=S>, S: Deref<Target=str> {
		trackers.into_iter().filter_map(	//todo: map to references instead of having tracker.deref()
			|tracker| self.matcher.of_trackers.as_ref().and_then(
				|matcher| matcher.find(tracker.deref()).nth(0)
			).map(
				|matched| self.rules[matched.pati].destination.as_path()
			)
		).nth(0)
	}
}


