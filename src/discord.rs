//! Discord API items.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Choice {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Option {
    #[serde(rename = "type")]
    pub typ: i8,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub choices: Vec<Choice>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<Option>,
}

pub fn set_true() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub application_id: String,
    pub guild_id: String,
    pub name: String,
    pub description: String,
    #[serde(default = "set_true")]
    pub default_permissions: bool,
    pub options: Vec<Option>,
}

// TODO not complete, just grabbing what I need.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Interaction {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub application_id: String,
    #[serde(rename = "type")]
    pub typ: i8,
    #[serde(default, skip_serializing_if = "std::option::Option::is_none")]
    pub data: std::option::Option<Data>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub guild_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub channel_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub token: String,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub version: i8,
}

fn is_zero(i: &i8) -> bool {
    *i == 0
}

// TODO not complete, just grabbing what I need.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Data {
    pub id: String,
    pub name: String,
    pub options: Vec<Choice>,
    #[serde(default)]
    pub custom_id: String,
    #[serde(default)]
    pub component_type: i64,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Response {
    #[serde(rename = "type")]
    pub typ: i8,
    #[serde(default, skip_serializing_if = "std::option::Option::is_none")]
    pub data: std::option::Option<DataResponse>,
}

// TODO not complete, just grabbing what I need.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct DataResponse {
    pub tts: bool,
    pub content: String,
}
