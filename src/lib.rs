use std::collections::HashMap;

pub mod discord;
pub mod pokemon;

use hyper::{body::Bytes, Body, Client, Method, Request, Uri};
use hyper_tls::HttpsConnector;
use lazy_static::lazy_static;
use serde_json::to_string;

lazy_static! {
	pub static ref DISCORD_API: String = "https://discord.com/api/".to_string();
	pub static ref CLIENT: Client<hyper_tls::HttpsConnector<hyper::client::connect::HttpConnector>, hyper::body::Body> = {
		let https = HttpsConnector::new();
		Client::builder().build::<_, hyper::Body>(https)
	};
	pub static ref POKEDEX_DEFINITION: discord::Command = discord::Command {
		id: "".to_string(),
		application_id: "".to_string(),
		guild_id: "".to_string(),
		name: "pokedex".to_string(),
		description: "Looks up a pokemon in the pokedex database.".to_string(),
		default_permissions: true,
		options: vec![discord::Option {
			name: "pokemon".to_string(),
			description: "The name of the pokemon to look up.".to_string(),
			typ: 3,
			required: true,
			choices: vec![],
			options: vec![],
		}]
	};
	pub static ref INTERACTIONS: HashMap<String, discord::Command> = {
		let mut m: HashMap<String, discord::Command> = HashMap::new();
		m.insert("pokedex".to_string(), POKEDEX_DEFINITION.deref().clone());
		m
	};
}

#[derive(Clone, Debug)]
pub enum Error {
	API(String),
	Internal(String),
	Decode(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match *self {
			Error::API(ref s) => write!(f, "api error: {}", s),
			Error::Internal(ref s) => write!(f, "internal error: {}", s),
			Error::Decode(ref s) => write!(f, "decode error: {}", s),
		}
	}
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct Interactions {
	guild_id: String,
	application_id: String,
	bot_token: String,
}

impl Interactions {
	pub fn new(guild_id: &str, application_id: &str, bot_token: &str) -> Self {
		Interactions {
			guild_id: guild_id.to_string(),
			application_id: application_id.to_string(),
			bot_token: bot_token.to_string(),
		}
	}

	pub async fn get_commands(&self) -> Result<Vec<discord::Command>> {
		let uri = format!(
			"{}/applications/{}/guilds/{}/commands",
			DISCORD_API.to_string(),
			self.application_id,
			self.guild_id
		);
		let uri = match uri.parse::<Uri>() {
			Ok(uri) => uri,
			Err(err) => return Err(Error::Internal(err.to_string())),
		};

		let request = match Request::builder()
			.method(Method::GET)
			.uri(uri)
			.header("Authorization", format!("Bot {}", self.bot_token))
			.body(Body::empty())
		{
			Ok(request) => request,
			Err(err) => return Err(Error::Internal(err.to_string())),
		};

		let res = match CLIENT.request(request).await {
			Ok(res) => res,
			Err(err) => return Err(Error::API(err.to_string())),
		};

		let status = res.status();

		let body = match hyper::body::to_bytes(res).await {
			Ok(body) => body,
			Err(err) => return Err(Error::API(err.to_string())),
		};

		if !status.is_success() {
			let body = match std::str::from_utf8(&body) {
				Ok(body) => body.to_string(),
				Err(e) => e.to_string(),
			};
			return Err(Error::API(format!("get http error: {} {}", status, body)));
		}

		match serde_json::from_slice::<Vec<discord::Command>>(&body) {
			Ok(cmds) => Ok(cmds),
			Err(err) => return Err(Error::Decode(err.to_string())),
		}
	}

	pub async fn update_commands(&self) -> Result<()> {
		let uri = format!(
			"{}/applications/{}/guilds/{}/commands",
			DISCORD_API.to_string(),
			self.application_id,
			self.guild_id
		);
		let uri = match uri.parse::<Uri>() {
			Ok(uri) => uri,
			Err(err) => return Err(Error::Internal(err.to_string())),
		};

		for (_, cmd) in INTERACTIONS.iter() {
			let body = match to_string(cmd) {
				Ok(body) => body,
				Err(err) => return Err(Error::Decode(err.to_string())),
			};
			let request = match Request::builder()
				.method(Method::POST)
				.uri(uri.clone())
				.header("content-type", "application/json")
				.header("Authorization", format!("Bot {}", self.bot_token))
				.body(Body::from(Bytes::from(body)))
			{
				Ok(request) => request,
				Err(err) => return Err(Error::Internal(err.to_string())),
			};

			let res = match CLIENT.request(request).await {
				Ok(res) => res,
				Err(err) => return Err(Error::API(err.to_string())),
			};

			let status = res.status();

			let body = match hyper::body::to_bytes(res).await {
				Ok(body) => body,
				Err(err) => return Err(Error::API(err.to_string())),
			};

			if !status.is_success() {
				let body = match std::str::from_utf8(&body) {
					Ok(body) => body.to_string(),
					Err(e) => e.to_string(),
				};
				return Err(Error::API(format!("get http error: {} {}", status, body)));
			}
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}
