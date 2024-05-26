use std::{collections::HashMap, fmt::{self, Debug, Display}, str::FromStr};

use serde::{de::Visitor, Deserialize, Serialize};

struct LevelsVisitor;

impl<'de> Visitor<'de> for LevelsVisitor {
    type Value = Levels;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Map from level ID to level")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>, {
        let mut res = HashMap::<String, Level>::new();
        while let Some((k, mut v)) = map.next_entry::<String, Level>()? {
            v.id = k.to_owned();
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
    pub links: Vec<Link>
}

mod defaults {
    pub fn copyright() -> String {
        "Powered by <a href=\"https://github.com/mallo-c/qs/\">qs</a>".to_string()
    }
    pub fn loading() -> String {
        "Loading...".to_string()
    }
    pub fn wrong_answer() -> String {
        "Wrong answer".to_string()
    }
    pub fn back() -> String {
        "Go back".to_string()
    }
}

#[derive(Deserialize, Clone)]
pub struct Strings {
    pub name: String,
    #[serde(default="defaults::wrong_answer")]
    pub wrong_answer: String,
    #[serde(default="defaults::loading")]
    pub loading: String,
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
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Next {
    pub caption: String,
    pub to: String,
}

#[derive(Debug)]
pub struct Error {
    msg: String
}

impl FromStr for Error {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Error { msg: s.to_owned() })
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        return Error{msg: value.to_owned()};
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        return Error{msg: value};
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "config read error: {}", self.msg)
    }
}

impl Config {
    pub fn from_string(s: &str) -> Result<Self, Error> {
        match serde_yml::from_str(s) {
            Ok(c) => Ok(c),
            Err(e) => Err(e.to_string().into()),
        }
    }
    pub fn from_file(name: &str) -> Result<Self, Error> {
        match std::fs::read_to_string(name) {
            Ok(s) => Config::from_string(&s),
            Err(e) => Err(e.to_string().into())
        }
    }
}