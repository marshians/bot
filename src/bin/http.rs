use futures::lock::Mutex;
use std::convert::TryInto;

use interactions::{discord, pokemon, Interactions};

#[macro_use]
extern crate rocket;
use clap::Parser;
use rocket::{http::Status, response::status, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use serde_json::from_slice;
use sodiumoxide::crypto::sign;
use sodiumoxide::crypto::sign::ed25519::{PublicKey, Signature};

struct SignatureHeaders {
    signature: String,
    timestamp: String,
}

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for SignatureHeaders {
    type Error = String;

    async fn from_request(
        req: &'r rocket::request::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let headers = req.headers();
        let signature = headers
            .get_one("X-Signature-Ed25519")
            .unwrap_or("")
            .to_string();
        let timestamp = headers
            .get_one("X-Signature-TimeStamp")
            .unwrap_or("")
            .to_string();

        rocket::request::Outcome::Success(SignatureHeaders {
            signature,
            timestamp,
        })
    }
}

#[post("/", data = "<msg>")]
async fn handler(
    msg: Vec<u8>,
    sigs: SignatureHeaders,
    pub_key: &State<Mutex<PublicKey>>,
    p: &State<Mutex<pokemon::PokeAPI>>,
) -> Result<Json<discord::Response>, status::Custom<String>> {
    // Verify message
    let pub_key = pub_key.lock().await;
    let sig = match decode_hex(&sigs.signature) {
        Ok(sig) => sig,
        Err(err) => return Err(status::Custom(Status::Unauthorized, err.to_string())),
    };
    let sig = Signature::new(match sig.try_into() {
        Ok(sig) => sig,
        Err(_) => {
            return Err(status::Custom(
                Status::InternalServerError,
                "signature not 64-bits".to_string(),
            ))
        }
    });
    let data: Vec<u8> = [sigs.timestamp.as_bytes(), &msg.clone()].concat();

    if !sign::verify_detached(&sig, &data, &pub_key) {
        return Err(status::Custom(
            Status::Unauthorized,
            "unable to verify signature".to_string(),
        ));
    };

    let msg: discord::Interaction = match from_slice(&msg) {
        Ok(msg) => msg,
        Err(err) => return Err(status::Custom(Status::BadRequest, err.to_string())),
    };

    if msg.typ == 1 {
        let mut response = discord::Response::default();
        response.typ = 1;

        return Ok(Json(response));
    } else {
        // We only have one option, so we only needs the first.
        let search = match msg.data {
            Some(data) => match data.options.first() {
                Some(option) => option.value.clone(),
                None => {
                    return Err(status::Custom(
                        Status::BadRequest,
                        "no option given".to_string(),
                    ))
                }
            },
            None => {
                return Err(status::Custom(
                    Status::BadRequest,
                    "no data given".to_string(),
                ))
            }
        };

        let p = p.lock().await;
        let pokemon = match p.get_pokemon(&search).await {
            Ok(pokemon) => pokemon,
            Err(err) => return Err(status::Custom(Status::InternalServerError, err)),
        };

        let mut response = discord::Response::default();
        response.typ = 4;
        response.data = Some(discord::DataResponse {
            tts: false,
            content: pokemon.markdown(),
        });
        return Ok(Json(response));
    }
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

#[derive(Parser)]
#[clap(version = "1.0", author = "Joshua Marsh <joshua@themarshians.com>")]
struct Options {
    /// The Discord Server ID.
    #[clap(short, long, default_value = "config.yaml")]
    pub config: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Config {
    guild_id: String,
    application_id: String,
    bot_token: String,
    public_key: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse our arguments and read our config.
    let opts = Options::parse();
    let config = std::fs::read_to_string(&opts.config)?;
    let config: Config = serde_yaml::from_str(&config)?;

    // Setup the public key we'll use for checking signatures.
    let pub_key = decode_hex(&config.public_key)?;
    let pub_key = PublicKey::from_slice(&pub_key).unwrap();

    // Setup our interactions.
    let i = Interactions::new(&config.guild_id, &config.application_id, &config.bot_token);

    // Setup out connection to the Pokemon API.
    let p = pokemon::PokeAPI::new().await;

    // Initialize commands.
    i.update_commands().await?;

    rocket::build()
        .manage(Mutex::new(p))
        .manage(Mutex::new(pub_key))
        .manage(Mutex::new(i))
        .mount("/", routes![handler])
        .launch()
        .await?;

    Ok(())
}
