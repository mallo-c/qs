use std::path::PathBuf;
use std::str::FromStr;
use std::{
    collections::HashMap,
    fmt::{self, Debug, Display},
    fs::File,
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
        let mut res = HashMap::new();
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
pub struct Attachments(pub HashMap<String, PathBuf>);

impl Deref for Attachments {
    type Target = HashMap<String, PathBuf>;
    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

impl<'de> Deserialize<'de> for Attachments {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(AttachmentsVisitor)
    }
}

struct AttachmentsVisitor;

impl AttachmentsVisitor {
    fn get_fname(e: impl ToString) -> Result<String, Box<dyn std::error::Error>> {
        Ok(
            e
                .to_string()
                .parse::<PathBuf>()?
                .canonicalize()?
                .file_name()
                .ok_or(format!(
                    "failed to get file name: {}",
                    e.to_string()
                ))?
                .to_str()
                .ok_or(format!(
                    "non-Unicode characters found: {}",
                    e.to_string()
                ))?
                .to_owned()
        )
    }
}

impl<'de> Visitor<'de> for AttachmentsVisitor {
    type Value = Attachments;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "list of files to expose")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut res = HashMap::new();
        while let Some(e) = seq.next_element::<String>()? {
            res.insert(
                AttachmentsVisitor::get_fname(&e)
                    .map_err(serde::de::Error::custom)?,
                e.parse()
                    .map_err(serde::de::Error::custom)?,
            );
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

#[derive(Deserialize, Clone, Default)]
#[serde(rename_all="lowercase")]
pub enum Key {
    Exact(String),
    Checker(String),
    #[default]
    None
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Level {
    #[serde(skip)]
    pub id: String,
    pub legend: String,
    pub next: Option<Next>,
    #[serde(default)]
    pub key: Key,
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
