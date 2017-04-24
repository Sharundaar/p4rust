extern crate itertools;

use itertools::Itertools;

use std::process::Command;
use std::process::Stdio;
use std::error::Error;
use std::io::Write;
use std::io::Read;

use std::vec::Vec;

use std::fmt;
use std::fmt::Formatter;
use std::fmt::Display;

// use std::convert::AsRef;
use std::ffi::OsStr;
use std::ffi::OsString;

enum P4Result {
	P4Success,
	P4Error (Vec<String>),
}

enum P4Changelist {
	Default,
	New,
	Change(u32),
}

struct P4Command {
	port: Option<OsString>,
	client: Option<OsString>,
	working_dir: Option<OsString>,
	changelist: P4Changelist,
	description: Option<OsString>,
	files: Vec<OsString>,
}

struct P4FileCommandResult {
	changelist: P4Changelist,
	success: Vec<(String, String)>,
	failure: Vec<(String, String)>,
}

struct P4ChangelistResult {
	changelist: P4Changelist,
	description: Option<String>,
	status: Option<String>,
	user: Option<String>,
	client: Option<String>,
	files: Option<Vec<(String, String)>>,
}

impl P4ChangelistResult {
	fn new() -> P4ChangelistResult {
		P4ChangelistResult {
			changelist: P4Changelist::Default,
			description: None,
			status: None,
			user: None,
			client: None,
			files: None,
		}		
	}

	fn from_read<T: std::io::Read>(source: &mut T) -> P4ChangelistResult {
		use P4Changelist::{ Default, Change };
		let mut result = P4ChangelistResult::new();

		let mut source_str = String::new();
		match source.read_to_string(&mut source_str) {
			Err(why) => println!("Error while reading from source: {}", why),
			_ => {},
		};
		let mut it = source_str.split("\n");
		
		while let Some(line) = it.next() {
			if line.len() == 0 {continue}; // skip empty lines
			if line.chars().nth(0) == Some('#') {continue}; // skip comments
			
			let split: Vec<&str> = line.split(":").collect();
			match split[0] {
				"Change" => result.changelist = Change(split[1].trim().parse::<u32>().unwrap()),
				
				"Client" => result.client = Some(split[1].trim().to_string()),

				"User" => result.user = Some(split[1].trim().to_string()),

				"Status" => result.status = Some(split[1].trim().to_string()),

				"Description" => {
					result.description = Some(it.take_while_ref(|&x| x.chars().nth(0) == Some('\t'))
											.map(|x| x.trim().to_string())
											.join("\n"));
				},

				"Files" => {
					result.files = Some(it.take_while_ref(|&x| x.chars().nth(0) == Some('\t'))
										.map(|x| {
											let split: Vec<&str> = x.split("#").collect();
											(split[0].trim().to_string(), split[1].trim().to_string())
										}).collect()
									);
				},
				
				_ => {continue},
			}
		}

		result
	}
}

impl P4Command {
	fn new() -> P4Command {
		P4Command {
			port: None,
			client: None,
			working_dir: None,
			changelist: P4Changelist::Default,
			description: None,
			files: Vec::new(),
		}
	}

	fn file(&mut self, file: &str) -> &mut P4Command {
		self.files.push(OsString::from(file));
		self
	}

	fn description(&mut self, desc: &str) -> &mut P4Command {
		self.description = Some(OsString::from(desc));
		self
	}

	fn client(&mut self, client: &str) -> &mut P4Command {
		self.client = Some(OsString::from(client));
		self
	}

	fn changelist(&mut self, changelist: P4Changelist) -> &mut P4Command {
		self.changelist = changelist;
		self
	}

	fn run_change(&mut self) -> Result<P4ChangelistResult, String> {
		use P4Changelist::{ Change, New, Default };

		let mut args = "/C p4 change ".to_string();
		let mut result = P4ChangelistResult::new();

		match self.changelist {
			Change(cl) => {
				args.push_str(&format!("-o {}", cl));
				let mut process = match Command::new("cmd")
											.arg(&args)
											.stdout(Stdio::piped())
											.stderr(Stdio::piped())
											.spawn() {
					Err(why) => return Err(format!("Failed to spawn process. Reason: {}", why)),
					Ok(process) => process,
				};

				if let Ok(exitStatus) = process.wait() {
					if exitStatus.success() {
							result = P4ChangelistResult::from_read(&mut process.stdout.unwrap());
					}
					else {
						let mut why = String::new();
						process.stderr.unwrap().read_to_string(&mut why);
						return Err(format!("Error in p4 change. Reason: {}", why));
					};
				};
			},

			New => {

			},

			Default => {

			},
		};

		Ok(result)
	}

/*
	fn run_edit(&mut self) -> P4FileCommandResult {
		
	}
	*/
}

fn main() {
	
	let result = P4Command::new()
					.changelist(P4Changelist::Change(39))
					.run_change();

	match result {
		Ok(res) => {
			println!("Ok!");

			if let P4Changelist::Change(cl) = res.changelist { println!("Changelist: {}", cl); }
			println!("Description: \n{}", res.description.unwrap());
			println!("Status: {}", res.status.unwrap());
			println!("User: {}", res.user.unwrap());
			println!("Client: {}", res.client.unwrap());
			println!("Files:");
			if let Some(files) = res.files {
				for (file, state) in files {
					println!("{}\t#\t{}", file, state);
				}
			}
			
		},
		Err(err) => println!("{}", err),
	}
}

static CHANGE: &'static str =
"Change: new\nDescription: random desc\n";

#[allow(dead_code)]
fn change_test() {
	let process = match Command::new("cmd")
						.args(&["/C", r"p4 change -i"])
						.stdin(Stdio::piped())
						.stdout(Stdio::piped())
						.stderr(Stdio::piped())
						.spawn() {
		Err(why) => panic!("Error while spawning process: {}", why.description()),
		Ok(process) => process,
	};

	match process.stdin.unwrap().write_all(CHANGE.as_bytes()) {
		Err(why) => panic!("Error while writing in stdin: {}", why.description()),
		Ok(_) => println!("Sent change to p4 change"),
	};

	let mut s = String::new();
	match process.stdout.unwrap().read_to_string(&mut s) {
		Err(why) => panic!("Error while reading stdout: {}", why.description()),
		Ok(_) => println!("p4 change responded with: \n{}", s),
	};

	s = String::new();
	match process.stderr.unwrap().read_to_string(&mut s) {
		Err(why) => panic!("Error while reading stderr: {}", why.description()),
		Ok(_) => println!("p4 erred with: \n{}", s),
	};
}

#[allow(dead_code)]
fn edit_test() {
	let process = Command::new("cmd")
    					.args(&["/C", r"p4 edit"])
    					.arg( r"D:\PerforceRoot\TestFolder\tesst_file.txt" )
    					.arg( r"D:\PerforceRoot\TestFolder\test_file.txt" )
    					.output();

    let output = match process {
    	Err(why) => panic!("An error occured in process: {}", why.description()),
    	Ok(output) => output,
    };

    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
}