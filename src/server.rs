use std::cell::RefCell;
use std::io::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::rc::Rc;
use mio::{Events, Poll, Token};
use crate::http::HttpProxy;
use crate::proxy::ProxySession;

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("could not create event loop")]
    CreatePoll(Error),
    // #[error("could not clone the MIO registry: {0}")]
    // CloneRegistry,
    // #[error("could not register the channel: {0}")]
    // RegisterChannel
    // #[error("{msg}:{scm_err}")]
    // ScmSocket {
    //     msg: String,
    //     scm_err: ScmSocketError,
    // },
}

pub struct Server {
    pub session_manager: SessionManager,
    http: HttpProxy,
    token_counter: i16,
    pub poll: Poll,
}

pub struct ListenSession {
    // pub protocol: Protocol,
}

impl ProxySession for ListenSession {}

impl Server {
    pub fn new() -> Result<Server, ServerError> {
        let event_loop = Poll::new().map_err(ServerError::CreatePoll)?;
        let registry = event_loop
            .registry()
            .try_clone()
            .map_err(ServerError::CreatePoll)?;

        let http = HttpProxy::new(registry);

        let sessions = SessionManager::new();
        let mut server = Server {
            session_manager: sessions,
            token_counter: 0,
            http,
            poll: event_loop,
        };
        server.add_listeners();
        Ok(server)
    }

    fn add_listeners(&mut self) {
        let listen_session = ListenSession {};
        self.session_manager.sessions.push(Rc::new(RefCell::new(listen_session)));
        self.token_counter += 1;

        let ip = Ipv4Addr::new(127, 0, 0, 1); // Localhost IP
        let port = 8080;                      // Port number
        let addr = SocketAddr::new(ip.into(), port); // Create SocketAddr

        let token = Token(self.token_counter as usize);
        self.http.add_listener(token);

        let activate_token = self.http.activate_listener(&addr);

    }

    pub fn run(&mut self) {
        println!("running");
        let mut events = Events::with_capacity(1024);
        loop {
            self.poll.poll(&mut events, None);

            for event in events.iter() {
                println!("{:?}", event.token())
            }
        }
    }
}

pub struct SessionManager {
    pub sessions: Vec<Rc<RefCell<dyn ProxySession>>>,
}

impl SessionManager {
    fn new() -> SessionManager {
        SessionManager {
            sessions: Vec::new()
        }
    }
}