extern crate bip_bencode;
use self::bip_bencode::Bencode; 

pub fn trackers(data: &Vec<u8>) -> Result<Vec<&str>, &'static str> {
	let decoded = match Bencode::decode(data.as_slice()) {
		Ok(d) => d,
		Err(_) => {
			return Err("corrupted file");
		},
	};

	//todo: make this readable oh god
	if let Some(trackers) = decoded.dict().and_then(|dict| dict.lookup("announce-list")).and_then(|decoded| decoded.list()).map(|list| list.iter().filter_map(|list| list.list())).map(|list| list.flat_map(|inner_list| inner_list.iter()).filter_map(|decoded| decoded.str()).collect()) {	//supports BEP 12
		Ok(trackers)
	} else if let Some(tracker) = decoded.dict().and_then(|dict| dict.lookup("announce")).and_then(|t| t.str()) {	//fallback
		let mut trackers = Vec::<&str>::new();
		trackers.reserve_exact(1);	//why waste memory
		trackers.push(tracker);
		Ok(trackers)
	} else {
		Err("malformed file")
	}
}
