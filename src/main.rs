mod types;
mod query;
mod io;
mod tests;

use serde::{Deserialize, Serialize};
use std::io::{Read, Result, Write};
use crate::query::query::QueryRunner;

fn main() -> Result<()> {
    QueryRunner::repl()?;
    Ok(())
}
