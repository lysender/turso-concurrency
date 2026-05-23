use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;

use crate::{Error, Result};
use std::env;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub db_dir: PathBuf,
    pub db_pooled: bool,
    pub write: bool,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CliArgs {
    #[arg(long)]
    pooled: bool,

    #[arg(long)]
    write: bool,
}

impl Config {
    pub fn build() -> Result<Self> {
        let cli = CliArgs::parse();
        let db_dir = PathBuf::from(required_env("DATABASE_DIR")?);

        Ok(Config {
            db_dir,
            db_pooled: cli.pooled,
            write: cli.write,
        })
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
