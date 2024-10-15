use std::net::SocketAddr;
use mio::net::TcpListener;
use socket2::{Domain, Protocol, Socket, Type};

#[derive(thiserror::Error, Debug)]
pub enum ServerBindError {
    #[error("could not set bind to socket: {0}")]
    BindError(std::io::Error),
    #[error("could not listen on socket: {0}")]
    Listen(std::io::Error),
    #[error("could not set socket to nonblocking: {0}")]
    SetNonBlocking(std::io::Error),
    #[error("could not set reuse address: {0}")]
    SetReuseAddress(std::io::Error),
    #[error("could not set reuse address: {0}")]
    SetReusePort(std::io::Error),
    #[error("Could not create socket: {0}")]
    SocketCreationError(std::io::Error),
    #[error("Invalid socket address '{address}': {error}")]
    InvalidSocketAddress { address: String, error: String },
}

pub fn server_bind(addr: SocketAddr) -> Result<TcpListener, ServerBindError> {
    let sock = Socket::new(Domain::for_address(addr), Type::STREAM, Some(Protocol::TCP))
        .map_err(ServerBindError::SocketCreationError)?;

    // set so_reuseaddr, but only on unix (mirrors what libstd does)
    if cfg!(unix) {
        sock.set_reuse_address(true)
            .map_err(ServerBindError::SetReuseAddress)?;
    }

    // sock.set_reuse_port(true)
    //     .map_err(ServerBindError::SetReusePort)?;

    sock.bind(&addr.into())
        .map_err(ServerBindError::BindError)?;

    sock.set_nonblocking(true)
        .map_err(ServerBindError::SetNonBlocking)?;

    // listen
    // FIXME: make the backlog configurable?
    sock.listen(1024).map_err(ServerBindError::Listen)?;

    Ok(TcpListener::from_std(sock.into()))
}