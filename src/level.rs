use std::{collections::HashMap, fmt::Display, sync::Arc};

use crate::config::{Attachment, Config};

pub struct LevelManager {
    st: HashMap<String, Arc<Level>>,
}

impl LevelManager {
    pub fn from_config(c: &Config) -> Result<Self, LevelInspectError> {
        let mut st = HashMap::new();
        Self::dfs(c, "start", None, &mut st)?;
        Ok(LevelManager {st})
    }
    fn dfs(c: &Config, st: &str, prev: Option<&str>, hm: &mut HashMap<String, Arc<Level>>) -> Result<(), LevelInspectError> {
        if prev.is_some_and(|x| x == st) {
            return Err(LevelInspectError::LoopDetected);
        }
        let lev = match c.levels.0.get(st) {
            None => return Err(LevelInspectError::NotFound(st.to_string())),
            Some(x) => x,
        };
        let lev = Level {
            id: lev.id.clone(),
            legend: lev.legend.clone(),
            next: match lev.next {
                None => Next::None,
                Some(crate::config::Next{ref to, ref caption}) => {
                    Self::dfs(c, to, Some(st), hm)?;
                    Next::Button { caption: caption.clone(), to: hm.get(to).ok_or(LevelInspectError::NotFound(to.to_string()))?.clone() }
                },
            },
            key: lev.key.clone(),
            attachments: lev.attachments.clone()
        };
        hm.insert(st.to_string(), Arc::new(lev));
        Ok(())
    }
    pub fn get(&self, id: &str) -> Option<Arc<Level>> {
        self.st.get(id).map(|v| v.clone())
    }
}

pub struct Level {
    pub id: String,
    pub legend: String,
    pub next: Next,
    pub key: Option<String>,
    pub attachments: Vec<Attachment>
}

pub enum Next {
    None,
    Button{caption: String, to: Arc<Level>}
}

#[derive(Debug)]
pub enum LevelInspectError {
    LoopDetected,
    NotFound(String),
}

impl Display for LevelInspectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LevelInspectError::LoopDetected => write!(f, "loop detected"),
            LevelInspectError::NotFound(i) => write!(f, "level not found: `{i}`"),
        }
    }
}
