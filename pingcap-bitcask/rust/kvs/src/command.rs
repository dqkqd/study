use std::{
    collections::BTreeMap,
    time::{Duration, SystemTime},
};

use crate::{log::LogId, parser::ByteParser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Command {
    Set {
        key: String,
        value: String,
        timestamp: Duration,
    },
    Remove {
        key: String,
        timestamp: Duration,
    },
}

impl ByteParser for Command {}

impl Command {
    pub fn set(key: String, value: String) -> Command {
        Command::Set {
            key,
            value,
            timestamp: current_timestamp(),
        }
    }

    pub fn remove(key: String) -> Command {
        Command::Remove {
            key,
            timestamp: current_timestamp(),
        }
    }

    pub fn key(&self) -> String {
        match self {
            Command::Set {
                key,
                value: _,
                timestamp: _,
            } => key.clone(),
            Command::Remove { key, timestamp: _ } => key.clone(),
        }
    }

    pub fn value(&self) -> Option<String> {
        match self {
            Command::Set {
                key: _,
                value,
                timestamp: _,
            } => Some(value.clone()),
            _ => None,
        }
    }

    pub fn timestamp(&self) -> Duration {
        match self {
            Command::Set {
                key: _,
                value: _,
                timestamp,
            } => *timestamp,
            Command::Remove { key: _, timestamp } => *timestamp,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CommandLocation {
    pub id: LogId,
    pub offset: usize,
    pub timestamp: Duration,
}

#[derive(Debug, Default)]
pub(crate) struct CommandLocations {
    pub data: BTreeMap<String, CommandLocation>,
}

impl CommandLocations {
    pub fn new() -> CommandLocations {
        CommandLocations::default()
    }

    pub fn merge(&mut self, key: String, location: CommandLocation) {
        self.data
            .entry(key)
            .and_modify(|old_location| {
                if old_location.timestamp < location.timestamp {
                    *old_location = location;
                }
            })
            .or_insert(location);
    }
}

pub(crate) fn current_timestamp() -> Duration {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("get duration since unix epoch")
}
