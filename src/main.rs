use crate::server::{Server, ServerError};

mod server;
mod proxy;
mod socket;
mod http;
mod log;
mod token_counter;

fn main() -> Result<(), ServerError> {
    // error!("this is error");
    let mut server = Server::new()?;
    server.run();
    Ok(())
}