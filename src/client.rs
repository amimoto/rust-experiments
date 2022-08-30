use serde_json::{ Value, to_vec, from_slice, json };

// To manage concurrency
use smol::Executor;
use std::sync::{Arc, Mutex};

pub mod transport;

const WAMP_HELLO:u64 = 1;
const WAMP_WELCOME:u64 = 2;
const WAMP_CHALLENGE:u64 = 4;
const WAMP_AUTHENTICATE:u64 = 5;

struct ConnectionInfo {
    url: String,
    realm: String,
    username: String,
    password: String,
}

struct Tracker {
}

#[derive(Clone)]
pub struct Wamp {
    info: Arc<ConnectionInfo>,
    transport: transport::Transport,
    tracker: Arc<Mutex<Tracker>>,
}

impl Wamp {

    pub async fn authenticate(&mut self) {
        let message = json!([
                            WAMP_HELLO,
                            self.info.realm,
                            {
                                "agent": "swampyer-rs",
                                "authid": self.info.username,
                                "authmethods": [ "ticket" ],
                                "roles": {
                                    "subscriber": {},
                                    "publisher": {},
                                    "caller": {},
                                    "callee": {},
                                }
                            }
                        ]);
        self.message_send(message).await;
    }

    pub async fn handle_challenge(&mut self, message:Value) {
        self.message_send(json!([
                            WAMP_AUTHENTICATE,
                            self.info.password,
                            {}
                        ])).await;
    }

    pub async fn handle_welcome(&mut self, message:Value) {
        println!("GOT WELCOME: {:?}", message);
    }

    pub async fn message_send(&mut self, message:Value) {
        self.transport.message_send(to_vec(&message).unwrap()).await;
    }

    pub async fn message_process(&mut self, message:Value) {
        println!("Parsed data {:?}", message);
        let message_type = message[0].as_u64().unwrap();
        match message_type {
            WAMP_CHALLENGE => {
                println!("authentication request");
                self.handle_challenge(message).await;
            },
            WAMP_WELCOME => {
                println!("welcome");
                self.handle_welcome(message).await;
            },
            _ => {
            },
        };
    }

    pub async fn message_get(&mut self) -> Option<Value> {
        if let Some(buf) = self.transport.message_get().await {
            let message:Value = from_slice(&buf).unwrap();
            return Some(message);
        }
        None
    }

    pub async fn connect(url:&str, realm:&str, username:&str, password:&str) -> Wamp {
        let info = ConnectionInfo {
                        url: url.to_string(),
                        realm: realm.to_string(),
                        username: username.to_string(),
                        password: password.to_string(),
                    };
        let transport = transport::Transport::connect("10.2.2.195:13780");
        let tracker = Tracker {};
        let mut wamp = Wamp {
            info: Arc::new(info),
            transport,
            tracker: Arc::new(Mutex::new(tracker)),
        };

        wamp.authenticate().await;

        wamp
    }

    pub async fn run(&mut self) {
        let ex = Executor::new();

        // Create the reader loop
        let mut reader_copy = self.clone();
        ex.run(async {
            loop {
                match reader_copy.message_get().await {
                    None => { println!("Timeout on read. Trying again." ) },
                    Some(message) => {
                        let mut handler_copy = reader_copy.clone();
                        let task = ex.spawn(async move {
                            handler_copy.message_process(message).await;
                        });
                        task.detach();
                    }
                }
            }
        }).await;
    }

    pub async fn call(&self, url:&str) {
    }
}
