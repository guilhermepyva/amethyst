use crate::game::player::PlayerList;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Poll, Token, Interest};
use std::time::Duration;
use std::net::{SocketAddr, Shutdown};
use mio::event::Source;

//Server address
const ADDR: &str = "127.0.0.1:25565";

//Token for epoll identification
const SERVER_TOKEN: Token = Token(0);

pub fn start(players: PlayerList) {
    //Open server
    let mut server = TcpListener::bind(ADDR.parse().unwrap()).expect("An error occured while binding the server");

    //Initialize epoll
    let mut poll = Poll::new().expect("An error occured while initializing the epoll");
    let mut events = Events::with_capacity(1024);

    //Register the server
    poll.registry().register(&mut server, SERVER_TOKEN, Interest::READABLE);

    //Client list
    let mut clients = Vec::new();
    let mut token_counter = 1usize;

    println!("Waiting for connections on {}", ADDR);

    std::thread::Builder::new().name("IO Network Thread".to_string()).spawn(move || {
        loop {
            //Poll events
            poll.poll(&mut events, None);

            if events.is_empty() {continue;}

            for event in events.iter() {
                let token = event.token();

                //Check server event
                if token == SERVER_TOKEN {
                    //Loop to accept all clients and finish server reading
                    loop {
                        match server.accept() {
                            //Got a client
                            Ok(mut client) => {
                                //Check if client is already logging
                                println!("{:?}", clients);
                                if clients.iter().any(|x: &(usize, TcpStream, SocketAddr) | client.1.ip().eq(&x.2.ip())) {
                                    //TODO Disconnect client
                                    println!("ih");
                                }

                                //Register client in poll and clients vector
                                poll.registry().register(&mut client.0, Token(token_counter), Interest::READABLE);
                                clients.push((token_counter, client.0, client.1));
                                token_counter += 1;

                                println!("Client connected: {}", client.1.ip())
                            }

                            //Check if it reached the end
                            Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => break,

                            //Check for another error
                            Err(e) => println!("An error occured while accepting a client: {}", e)
                        }
                    }
                } else {
                    //Check for clients token
                    let client = clients.iter_mut().find(|client| client.0 == token.0);

                    if client.is_some() {
                        let mut client = client.unwrap();
                    }
                }
            }
        }
    });
}