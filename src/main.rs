use crate::server::{Server, ServerError};

mod server;
mod proxy;
mod socket;
mod http;

fn main() -> Result<(), ServerError>{
    let mut server = Server::new()?;
    server.run();
    Ok(())
}