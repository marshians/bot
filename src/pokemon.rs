//! PokeAPI access.
use hyper::{Body, Client, Method, Request, Uri};
use hyper_tls::HttpsConnector;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
    pub static ref POKE_API: String = "https://pokeapi.co/api/v2".to_string();
    pub static ref CLIENT: Client<hyper_tls::HttpsConnector<hyper::client::connect::HttpConnector>, hyper::body::Body> = {
        let https = HttpsConnector::new();
        Client::builder().build::<_, hyper::Body>(https)
    };
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Type {
    pub name: String,
    pub url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TypeSlot {
    pub slot: u8,
    #[serde(rename = "type")]
    pub typ: Type,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sprites {
    pub front_default: Option<String>,
    pub front_shiny: Option<String>,
    pub front_female: Option<String>,
    pub front_shiny_female: Option<String>,
    pub back_default: Option<String>,
    pub back_shiny: Option<String>,
    pub back_female: Option<String>,
    pub back_shiny_female: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pokemon {
    pub name: String,
    pub sprites: Sprites,
    pub types: Vec<TypeSlot>,
    pub height: u64,
    pub weight: u64,
}

impl Pokemon {
    pub fn markdown(&self) -> String {
        format!(
            "![{}]({})
```
Name:   {}
Types:  {}
Height: {} decimeters
Weight: {} hectograms
```
",
            self.name,
            self.front(),
            self.name,
            self.types(),
            self.height,
            self.weight,
        )
    }

    pub fn types(&self) -> String {
        self.types
            .iter()
            .map(|t| t.typ.name.clone())
            .fold(String::new(), |a, b| a + &b + &", ".to_string())
            .trim_end_matches(", ")
            .to_string()
    }

    pub fn front(&self) -> String {
        self.sprites
            .front_default
            .as_ref()
            .unwrap_or(
                &"https://static.wikia.nocookie.net/pokemon-fano/images/6/6f/Poke_Ball.png"
                    .to_string(),
            )
            .clone()
    }
}

pub struct PokeAPI {}

impl PokeAPI {
    pub async fn new() -> Self {
        Self {}
    }

    pub async fn get_pokemon(&self, name: &str) -> Result<Pokemon, String> {
        let uri = format!("{}/pokemon/{}", POKE_API.to_string(), name);
        let uri = match uri.parse::<Uri>() {
            Ok(uri) => uri,
            Err(err) => return Err(err.to_string()),
        };

        let request = match Request::builder()
            .method(Method::GET)
            .uri(uri)
            .body(Body::empty())
        {
            Ok(request) => request,
            Err(err) => return Err(err.to_string()),
        };

        let res = match CLIENT.request(request).await {
            Ok(res) => res,
            Err(err) => return Err(err.to_string()),
        };

        let status = res.status();

        let body = match hyper::body::to_bytes(res).await {
            Ok(body) => body,
            Err(err) => return Err(err.to_string()),
        };

        if !status.is_success() {
            let body = match std::str::from_utf8(&body) {
                Ok(body) => body.to_string(),
                Err(e) => e.to_string(),
            };
            return Err(format!("get http error: {} {}", status, body));
        }

        match serde_json::from_slice::<Pokemon>(&body) {
            Ok(cmds) => Ok(cmds),
            Err(err) => Err(err.to_string()),
        }
    }
}
