use crate::query::runner::QueryRunner;

mod types;
mod query;
mod io;
mod tests;
mod rqle;

fn main() -> Result<(), ()> {
    QueryRunner::repl().expect("TODO: panic message");
    Ok(())
}
