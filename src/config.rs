use serde::Deserialize;
use std::path::PathBuf;

use crate::{Error, Result};
use std::env;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub db_dir: PathBuf,
}

impl Config {
    pub fn build() -> Result<Self> {
        let db_dir = PathBuf::from(required_env("DATABASE_DIR")?);

        Ok(Config { db_dir })
    }
}

fn required_env(name: &str) -> Result<String> {
    match env::var(name) {
        Ok(val) => {
            if val.is_empty() {
                return Err(Error::Config {
                    msg: format!("{} is required.", name),
                });
            }
            Ok(val)
        }
        Err(_) => Err(Error::Config {
            msg: format!("{} is required.", name),
        }),
    }
}
