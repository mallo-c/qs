use std::{collections::HashMap, fmt::{self, Debug, Display}, ops::Deref};
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

impl Deref for Levels {
    type Target = HashMap<String, Level>;
    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

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
    pub attachments: HashMap<String, String>,
    #[serde(default="defaults::start_level")]
    pub start: String,
    #[serde(default)]
    pub colors: Colors
}

#[derive(Deserialize, Clone)]
pub struct Colors {
    #[serde(default="defaults::color_primary")]
    pub primary: String,
    #[serde(default="defaults::color_secondary")]
    pub secondary: String,
    #[serde(default="defaults::color_background")]
    pub background: String
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            primary: defaults::color_primary(),
            secondary: defaults::color_secondary(),
            background: defaults::color_background()
        }
    }
}

mod defaults {
    pub fn wrong_answer() -> String {
        "Wrong answer".to_string()
    }
    pub fn back() -> String {
        "Go back".to_string()
    }
    pub fn start_level() -> String {
        "start".to_string()
    }
    pub fn not_found() -> String {
        "404 Level not found".to_string()
    }
    pub fn sanitize_legend() -> bool {
        true
    }
    pub fn color_primary() -> String {
        "#ffffff".to_owned()
    }
    pub fn color_secondary() -> String {
        "#00ffff".to_owned()
    }
    pub fn color_background() -> String {
        "#ffffff".to_owned()
    }
}

#[derive(Deserialize, Clone)]
pub struct Strings {
    pub name: String,
    #[serde(default="defaults::wrong_answer")]
    pub wrong_answer: String,
    #[serde(default="defaults::back")]
    pub back: String,
    #[serde(default="defaults::not_found")]
    pub not_found: String,
}

#[derive(Deserialize, Clone)]
pub struct Level {
    #[serde(skip)]
    pub id: String,
    pub legend: String,
    #[serde(default="defaults::sanitize_legend")]
    pub sanitize_legend: bool,
    pub next: Option<Next>,
    pub key: Option<String>,
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