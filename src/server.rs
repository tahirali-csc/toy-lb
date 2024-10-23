use std::cell::{Ref, RefCell};
use std::collections::{HashMap, VecDeque};
use std::io::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::rc::Rc;
use mio::{Events, Poll, Token};
use mio::event::Event;
use mio::net::TcpStream;
use crate::http::{AcceptError, HttpProxy};
use crate::proxy::ProxySession;
use crate::token_counter::TokenCounter;

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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListenToken(pub usize);

pub struct Server {
    pub session_manager: Rc<RefCell<SessionManager>>,
    http: Rc<RefCell<HttpProxy>>,
    accept_queue: VecDeque<(TcpStream, ListenToken)>,
    pub poll: Poll,
    token_counter: Rc<RefCell<TokenCounter>>
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

        let token_counter = Rc::new(RefCell::new(TokenCounter::new()));
        let sessions = Rc::new(RefCell::new(SessionManager::new()));

        let http = Rc::new(RefCell::new(HttpProxy::new(registry, sessions.clone(), token_counter.clone())));
        let mut server = Server {
            session_manager: sessions,
            token_counter: token_counter,
            http,
            poll: event_loop,
            accept_queue: VecDeque::new()
        };
        server.add_listeners();
        Ok(server)
    }

    fn add_listeners(&mut self) {
        let listen_session = ListenSession {};

        let token = self.token_counter.borrow_mut().next();
        let token = Token(token as usize);

        self.session_manager.borrow_mut().sessions.insert(token, Rc::new(RefCell::new(listen_session)));

        let ip = Ipv4Addr::new(127, 0, 0, 1); // Localhost IP
        let port = 8080;                      // Port number
        let addr = SocketAddr::new(ip.into(), port); // Create SocketAddr


        self.http.borrow_mut().add_listener(token);

        let activate_token = self.http.borrow_mut().activate_listener(&addr);

    }

    pub fn run(&mut self) {
        println!("running");
        let mut events = Events::with_capacity(1024);
        loop {
            self.poll.poll(&mut events, None);

            for event in events.iter() {
                println!("{:?}", event.token());
                self.ready(event.token(), event);
            }

            self.create_sessions()
        }
    }

    pub fn ready(&mut self, token: Token, event: &Event) {
        let session_token = token.0;
        if self.session_manager.borrow_mut().sessions.contains_key(&token) {
            if event.is_readable() {
                self.accept(ListenToken(token.0))
                // println!("is_readable")
            }
        }
    }

    pub fn accept(&mut self, token: ListenToken) {
        loop {
            // match self.http.borrow_mut().accept(token) {
            match self.http.borrow_mut().accept(token) {
                Ok(sock) => {
                    self.accept_queue.push_back((sock, token))
                }
                Err(AcceptError::WouldBlock) => {}
                Err(other) => {}
            }
        }
    }

    pub fn create_sessions(&mut self) {
        while let Some((sock, token)) = self.accept_queue.pop_back() {
            // if self.session_manager.borrow_mut().check_limits() {
            //
            // }
            // let proxy = self.http.clone();
            let  proxy = &mut self.http;
            proxy.borrow_mut().create_session(sock, token);
        }
    }
}

pub struct SessionManager {
    // pub sessions: Vec<Rc<RefCell<dyn ProxySession>>>,
    pub sessions: HashMap<Token, Rc<RefCell<dyn ProxySession>>>
}

impl SessionManager {
    fn new() -> SessionManager {
        SessionManager {
            sessions: HashMap::new()
        }
    }
}