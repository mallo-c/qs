use std::{collections::{HashMap, HashSet}, fmt::Display, sync::Arc};

use crate::config::{Attachment, Config};

pub struct LevelManager {
    st: HashMap<String, Arc<Level>>,
}

impl LevelManager {
    pub fn from_config(c: &Config) -> Result<Self, LevelInspectError> {
        let mut st = HashMap::new();
        let mut visited = HashSet::new();
        Self::dfs(c, &c.start, &mut st, &mut visited)?;
        Ok(LevelManager {st})
    }
    fn dfs(c: &Config, st: &str, hm: &mut HashMap<String, Arc<Level>>, vis: &mut HashSet<String>) -> Result<(), LevelInspectError> {
        if vis.contains(st) {
            return Err(LevelInspectError::LoopDetected);
        }
        vis.insert(st.to_string());
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
                    Self::dfs(c, to, hm, vis)?;
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
