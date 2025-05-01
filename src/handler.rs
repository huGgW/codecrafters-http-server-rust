use std::collections::HashMap;

use crate::{
	request::Request,
	response::{Response, Status},
	Args,
};

pub type Handler = dyn FnOnce(&Request) -> Result<Response, std::io::Error>;

pub fn default_handler(_: &Request) -> Result<Response, std::io::Error> {
	Ok(Response {
		status: Status {
			version: "1.1".to_string(),
			status_code: 200,
			status: "OK".to_string(),
		},
		headers: HashMap::new(),
		body: Vec::new(),
	})
}

pub fn unknwon_handler(_: &Request) -> Result<Response, std::io::Error> {
	Ok(Response {
		status: Status {
			version: "1.1".to_string(),
			status_code: 404,
			status: "Not Found".to_string(),
		},
		headers: HashMap::new(),
		body: Vec::new(),
	})
}

pub fn echo_handler(request: &Request) -> Result<Response, std::io::Error> {
	let echo_paths = request
		.start_line
		.path
		.split('/')
		.skip(2)
		.collect::<Vec<_>>();

	// we care only first element
	let echo_str = match echo_paths[..] {
		[s] => s,
		_ => {
			return Err(std::io::Error::new(
				std::io::ErrorKind::InvalidData,
				"not valid echo path given",
			))
		}
	};

	Ok(Response {
		status: Status {
			version: "1.1".to_string(),
			status_code: 200,
			status: "OK".to_string(),
		},
		headers: HashMap::from([
			(String::from("Content-Type"), String::from("text/plain")),
			(String::from("Content-Length"), echo_str.len().to_string()),
		]),
		body: echo_str.as_bytes().to_vec(),
	})
}

pub fn user_agent_handler(request: &Request) -> Result<Response, std::io::Error> {
	let user_agent = match request.headers.get("User-Agent".to_lowercase().as_str()) {
		Some(s) => s,
		None => {
			return Err(std::io::Error::new(
				std::io::ErrorKind::InvalidInput,
				"not valid user agent header given",
			))
		}
	};

	Ok(Response {
		status: Status {
			version: "1.1".to_string(),
			status_code: 200,
			status: "OK".to_string(),
		},
		headers: HashMap::from([
			(String::from("Content-Type"), String::from("text/plain")),
			(String::from("Content-Length"), user_agent.len().to_string()),
		]),
		body: user_agent.as_bytes().to_vec(),
	})
}

pub fn files_read_handler_provider(
	args: &Args,
) -> impl FnOnce(&Request) -> Result<Response, std::io::Error> {
	let directory_arg = args.directory.clone();

	move |request: &Request| -> Result<Response, std::io::Error> {
		let directory = match directory_arg {
			Some(d) => d.clone(),
			None => {
				return Err(std::io::Error::new(
					std::io::ErrorKind::NotFound,
					"directory arg not given",
				))
			}
		};

		let filename = match request.start_line.path[1..].split_once('/') {
			Some(("files", filename)) => filename,
			_ => {
				return Err(std::io::Error::new(
					std::io::ErrorKind::InvalidInput,
					"file path is not valid",
				))
			}
		};

		let file = std::fs::read(directory + filename)?;

		Ok(Response {
			status: Status {
				version: String::from("1.1"),
				status_code: 200,
				status: String::from("OK"),
			},
			headers: HashMap::from([
				(
					String::from("Content-Type"),
					String::from("application/octet-stream"),
				),
				(String::from("Content-Length"), file.len().to_string()),
			]),
			body: file,
		})
	}
}

pub fn files_write_handler_provider(
	args: &Args,
) -> impl FnOnce(&Request) -> Result<Response, std::io::Error> {
	let directory_arg = args.directory.clone();

	move |request: &Request| -> Result<Response, std::io::Error> {
		let directory = match directory_arg {
			Some(d) => d.clone(),
			None => {
				return Err(std::io::Error::new(
					std::io::ErrorKind::NotFound,
					"directory arg not given",
				))
			}
		};

		let filename = match request.start_line.path[1..].split_once('/') {
			Some(("files", filename)) => filename,
			_ => {
				return Err(std::io::Error::new(
					std::io::ErrorKind::InvalidInput,
					"file path is not valid",
				))
			}
		};

		std::fs::write(directory + filename, &request.body)?;

		Ok(Response {
			status: Status {
				version: String::from("1.1"),
				status_code: 201,
				status: String::from("Created"),
			},
			headers: HashMap::new(),
			body: Vec::new(),
		})
	}
}
