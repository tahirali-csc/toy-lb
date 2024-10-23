use crate::socket::server_bind;
use mio::net::{TcpListener, TcpStream};
use mio::{Interest, Registry, Token};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::rc::Rc;
use std::time::Duration;
use crate::proxy::ProxySession;
use crate::server::{ListenToken, SessionManager};
use crate::token_counter::TokenCounter;

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

#[derive(Debug, PartialEq, Eq)]
pub enum AcceptError {
    IoError,
    TooManySessions,
    WouldBlock,
    RegisterError,
    WrongSocketAddress,
    BufferCapacityReached,
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

    fn accept(&mut self) -> Result<TcpStream, AcceptError>{
        if let Some(ref sock) = self.listener {
            sock.accept()
                .map_err(|e| match e.kind() {
                    ErrorKind::WouldBlock => AcceptError::WouldBlock,
                    _ => {
                        // error!("accept() IO error: {:?}", e);
                        println!("accept() IO error: {:?}", e);
                        AcceptError::IoError
                    }
                })
                .map(|(sock, _)| sock)
        } else {
            println!("cannot accept connections, no listening socket available");
            Err(AcceptError::IoError)
        }
    }
}

pub struct HttpProxy {
    listeners: HashMap<Token, Rc<RefCell<HttpListener>>>,
    registry: Registry,
    sessions: Rc<RefCell<SessionManager>>,
    pub token_counter: Rc<RefCell<TokenCounter>>,
}

impl HttpProxy {
    pub fn create_session(&self, mut frontend_sock: TcpStream, listener_token: ListenToken) -> Result<(), AcceptError> {
        let listener = self
            .listeners
            .get(&Token(listener_token.0))
            .cloned()
            .ok_or(AcceptError::IoError)?;

        if let Err(e) = frontend_sock.set_nodelay(true) {
            return Err(AcceptError::IoError);
        }

        let mut session_manager = self.sessions.borrow_mut();
        let session_token = Token(self.token_counter.borrow_mut().next() as usize);
        // session_manager.sessions.insert()

        if let Err(err) = self.registry.register(&mut frontend_sock, session_token, Interest::READABLE | Interest::WRITABLE) {
            return Err(AcceptError::RegisterError);
        }

        // let owned_listener = listener.borrow();

        let session = HttpSession::new(
            Duration::from_secs(10),
            Duration::from_secs(10),
            Duration::from_secs(10),
            Duration::from_secs(10),
            session_token)?;

        let session = Rc::new(RefCell::new(session));
        session_manager.sessions.insert(session_token, session);
        Ok(())
    }
}

impl HttpProxy {
    pub fn new(
        registry: Registry,
        sessions: Rc<RefCell<SessionManager>>,
        token_counter: Rc<RefCell<TokenCounter>>
    ) -> HttpProxy {
        HttpProxy {
            listeners: HashMap::new(),
            registry,
            sessions,
            token_counter
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

    pub fn accept(&mut self, token: ListenToken) -> Result<TcpStream, AcceptError> {
        if let Some(listener) = self.listeners.get(&Token(token.0)) {
            listener.borrow_mut().accept()
        } else {
            Err(AcceptError::IoError)
        }
    }
}

pub struct HttpSession {
    // answers: Rc<RefCell<HttpAnswers>>,
    configured_backend_timeout: Duration,
    configured_connect_timeout: Duration,
    configured_frontend_timeout: Duration,
    frontend_token: Token,
}

impl HttpSession {
    pub fn new(
        configured_backend_timeout: Duration,
        configured_connect_timeout: Duration,
        configured_frontend_timeout: Duration,
        configured_request_timeout: Duration,
        token: Token,)-> Result<Self, AcceptError> {

        Ok(HttpSession {
            configured_backend_timeout,
            configured_connect_timeout,
            configured_frontend_timeout,
            frontend_token: token,
        })
    }
}

impl ProxySession for HttpSession {

}