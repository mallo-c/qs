use std::str::FromStr;
use std::{
    collections::HashMap,
    fmt::{self, Debug, Display},
    fs::{read_dir, File},
    io::{self, Read},
    ops::Deref,
    path::Path,
};

use serde::{de::Visitor, Deserialize, Serialize};

struct LevelsVisitor;

impl<'de> Visitor<'de> for LevelsVisitor {
    type Value = Levels;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Map from level ID to level configuration")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
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
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(LevelsVisitor)
    }
}

#[derive(Default, Clone)]
pub struct Attachments(pub HashMap<String, String>);

impl Deref for Attachments {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

impl<'de> Deserialize<'de> for Attachments {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(AttachmentsVisitor)
    }
}

struct AttachmentsVisitor;

impl<'de> Visitor<'de> for AttachmentsVisitor {
    type Value = Attachments;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "mapping from ID to path")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut res = HashMap::<String, String>::new();
        while let Some((k, v)) = map.next_entry::<String, String>()? {
            if k != "_" {
                res.insert(k, v);
            } else {
                match read_dir(&v) {
                    Ok(d) => {
                        for entry in d {
                            match entry {
                                Err(e) => {
                                    return Err(serde::de::Error::custom(format!(
                                        "failed to read dir {v}: {e}"
                                    )))
                                }
                                Ok(r) => {
                                    res.insert(
                                        r.file_name().to_str().unwrap().into(),
                                        r.path().to_str().unwrap().into(),
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        return Err(serde::de::Error::custom(format!(
                            "failed to read dir {v}: {e}"
                        )))
                    }
                }
            }
        }
        Ok(Attachments(res))
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
    pub attachments: Attachments,
    #[serde(default = "defaults::start_level")]
    pub start: String,
    #[serde(default)]
    pub colors: Colors,
}

#[derive(Deserialize, Clone)]
pub struct Colors {
    #[serde(default = "defaults::color_primary")]
    pub primary: String,
    #[serde(default = "defaults::color_secondary")]
    pub secondary: String,
    #[serde(default = "defaults::color_background")]
    pub background: String,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            primary: defaults::color_primary(),
            secondary: defaults::color_secondary(),
            background: defaults::color_background(),
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
    #[serde(default = "defaults::wrong_answer")]
    pub wrong_answer: String,
    #[serde(default = "defaults::back")]
    pub back: String,
    #[serde(default = "defaults::not_found")]
    pub not_found: String,
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
pub enum Error {
    ConfigParseError(serde_yml::Error),
    ReadError(std::io::Error),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Error::ConfigParseError(e) => write!(f, "could not parse config: {}", e),
            Error::ReadError(e) => write!(f, "could not read: {}", e),
        }
    }
}

impl FromStr for Config {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_yml::from_str(s).map_err(Error::ConfigParseError)
    }
}

impl Config {
    pub fn from_reader<R: Read>(f: R) -> Result<Self, Error> {
        io::read_to_string(f).map_err(Error::ReadError)?.parse()
    }
    pub fn from_path<P: AsRef<Path>>(p: P) -> Result<Self, Error> {
        Self::from_reader(File::open(p).map_err(Error::ReadError)?)
    }
}
