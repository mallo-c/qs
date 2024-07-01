use std::{collections::HashMap, fmt::{self, Debug, Display}};
use std::str::FromStr;

use serde::{de::Visitor, Deserialize, Serialize};

struct LevelsVisitor;

impl<'de> Visitor<'de> for LevelsVisitor {
    type Value = Levels;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Map from level ID to level configuration")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>, {
        let mut res = HashMap::<String, Level>::new();
        while let Some((k, mut v)) = map.next_entry::<String, Level>()? {
            k.clone_into(&mut v.id);
            res.insert(k, v);
        }
        Ok(Levels(res))
    }
}

#[derive(Clone)]
pub struct Levels(pub HashMap<String, Level>);

impl<'de> Deserialize<'de> for Levels {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        deserializer.deserialize_map(LevelsVisitor)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Link {
    pub caption: String,
    pub to: String,
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub strings: Strings,
    pub levels: Levels,
    #[serde(default)]
    pub links: Vec<Link>,
    #[serde(default)]
    pub attachments: HashMap<String, String>,
    #[serde(default="defaults::start_level")]
    pub start: String
}

mod defaults {
    pub fn copyright() -> String {
        "Powered by qs".to_string()
    }
    pub fn wrong_answer() -> String {
        "Wrong answer".to_string()
    }
    pub fn back() -> String {
        "Go back".to_string()
    }
    pub fn start_level() -> String {
        "start".to_string()
    }
    pub fn download() -> bool {
        false
    }
}

#[derive(Deserialize, Clone)]
pub struct Strings {
    pub name: String,
    #[serde(default="defaults::wrong_answer")]
    pub wrong_answer: String,
    #[serde(default="defaults::copyright")]
    pub copyright: String,
    #[serde(default="defaults::back")]
    pub back: String
}

#[derive(Deserialize, Clone)]
pub struct Level {
    #[serde(skip)]
    pub id: String,
    pub legend: String,
    pub next: Option<Next>,
    pub key: Option<String>,
    #[serde(default)]
    pub attachments: Vec<Attachment>
}

#[derive(Deserialize, Clone)]
pub struct Attachment {
    pub name: String,
    #[serde(default)]
    pub icon: Icon,
    pub file: String,
    #[serde(default="defaults::download")]
    pub download: bool,
}

#[derive(Deserialize, Clone, Default)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
pub enum Icon {
    #[default]
    File,
    Link,
    Download,
    Image,
    Media,
    Text,
    Archive,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Next {
    pub caption: String,
    pub to: String,
}

#[derive(Debug)]
pub enum Error {
    ConfigParseError(serde_yml::Error),
    FileReadError(std::io::Error)
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Error::ConfigParseError(e) => write!(f, "could not parse config: {}", e),
            Error::FileReadError(e) => write!(f, "could not open file: {}", e)
        }
    }
}

impl FromStr for Config {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_yml::from_str(s)
            .map_err(Error::ConfigParseError)
    }
}

impl Config {
    pub fn from_file(name: &str) -> Result<Self, Error> {
        std::fs::read_to_string(name)
            .map_err(Error::FileReadError)
            .and_then(|c| c.parse())
    }
}