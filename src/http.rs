use crate::socket::server_bind;
use mio::net::TcpListener;
use mio::{Interest, Registry, Token};
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::rc::Rc;

#[derive(thiserror::Error, Debug)]
pub enum ListenerError {
    #[error("failed to handle certificate request, got a resolver error, {0}")]
    // Resolver(CertificateResolverError),
    // #[error("failed to parse pem, {0}")]
    // PemParse(String),
    // #[error("failed to parse template {0}: {1}")]
    // TemplateParse(u16, TemplateError),
    // #[error("failed to build rustls context, {0}")]
    BuildRustls(String),
    #[error("could not activate listener with address {address:?}: {error}")]
    Activation { address: SocketAddr, error: String },
    #[error("Could not register listener socket: {0}")]
    SocketRegistration(std::io::Error),
    // #[error("could not add frontend: {0}")]
    // AddFrontend(RouterError),
    // #[error("could not remove frontend: {0}")]
    // RemoveFrontend(RouterError),
}

#[derive(thiserror::Error, Debug)]
pub enum ProxyError {
    #[error("failed to activate listener with address {address:?}: {listener_error}")]
    ListenerActivation {
        address: SocketAddr,
        listener_error: ListenerError,
    },
    #[error("found no listener with address {0:?}")]
    NoListenerFound(SocketAddr),
}
pub struct HttpListener {
    address: SocketAddr,
    listener: Option<TcpListener>,
    token: Token,
}

impl HttpListener {
    fn activate(&mut self, registry: &Registry) -> Result<Token, ListenerError> {
        let ipv4_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let mut listener = server_bind(ipv4_addr).map_err(|server_bind_err| ListenerError::Activation {
            address: ipv4_addr,
            error: String::from("error in binding"),
        })?;

        // println!("Token---{:?}", self.token);
        registry.register(&mut listener, self.token, Interest::READABLE).
            map_err(ListenerError::SocketRegistration)?;

        self.listener = Some(listener);
        Ok(self.token)
    }
}

pub struct HttpProxy {
    listeners: HashMap<Token, Rc<RefCell<HttpListener>>>,
    registry: Registry,
}


impl HttpProxy {
    pub fn new(registry: Registry) -> HttpProxy {
        HttpProxy {
            listeners: HashMap::new(),
            registry
        }
    }

    pub fn add_listener(&mut self, token: Token) {
        let ipv4_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        // self.listeners.entry(token);
        // self.listeners.entry(token)
        self.listeners.insert(token, Rc::new(RefCell::new(HttpListener{
            address:  ipv4_addr,
            token,
            listener: None,
        })));
    }

    pub fn activate_listener(&self, addr: &SocketAddr) -> Result<Token, ProxyError> {
        let listener = self
            .listeners
            .values()
            .find(|listener| listener.borrow().address == *addr)
            .ok_or(ProxyError::NoListenerFound(addr.to_owned()))?;

        listener
            .borrow_mut()
            .activate(&self.registry)
            .map_err(|listener_error| ProxyError::ListenerActivation {
                address: *addr,
                listener_error
            })
    }
}