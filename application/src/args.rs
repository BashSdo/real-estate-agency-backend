//! [`Args`] definitions.

use clap::Parser;

/// Server of the realty management system.
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to the configuration file.
    #[arg(short, long, default_value = "config.toml")]
    pub config: String,
}

impl Args {
    /// Parses command line arguments.
    ///
    /// # Errors
    ///
    /// Errors if failed to parse command line arguments.
    pub fn parse() -> Result<Self, clap::Error> {
        <Self as Parser>::try_parse()
    }
}
