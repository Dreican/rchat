use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};
use std::str;

const IP: &str = "0.0.0.0";
const PORT: &str = "5000";
const MESSAGE_RATE: Duration = Duration::from_secs(1);

enum Message {
    ClientConnected {
        author: Arc<TcpStream>,
    },
    ClientDisconnected {
        author_addr: SocketAddr,
    },
    NewMessage {
        author_addr: SocketAddr,
        bytes: Vec<u8>,
    },
}

struct Client {
    stream: Arc<TcpStream>,
    last_message: SystemTime,
}

fn main() -> Result<(), ()>
{
    let address = format!("{IP}:{PORT}");
    let listener = TcpListener::bind(&address)
        .map_err(|e|
        {
            eprintln!("ERROR: Unable to bind server on {address} with error: {e}");
        })?;
    println!("INFO: Listening on {}", &address);

    let (sender, receiver) = channel();
    thread::spawn(|| server(receiver));

    for stream in listener.incoming()
    {
        match stream
        {
            Ok(stream) =>
            {
                let stream = Arc::new(stream);
                let message_sender = sender.clone();
                thread::spawn(|| client(stream, message_sender));
            }
            Err(e) =>
            {
                eprintln!("ERROR: Connection failed {}", e)
            }
        }
    }
    drop(listener);
    Ok(())
}

fn server(messages: Receiver<Message>) -> Result<(), ()>
{
    let mut clients = HashMap::<SocketAddr, Client>::new();
    loop
    {
        let msg = messages.recv().expect("The server receiver is down");
        match msg
        {
            Message::ClientConnected { author } =>
            {
                let author_addr = author.peer_addr().expect("No peer address");
                let now = SystemTime::now();
                println!("INFO: Client {author_addr} connected");
                clients.insert(author_addr.clone(), Client
                    {
                        stream: author.clone(),
                        last_message: now - 2 * MESSAGE_RATE,
                    },
                );
            }
            Message::ClientDisconnected { author_addr } =>
            {
                println!("INFO: Client {author_addr} disconnected");
                clients.remove(&author_addr);
            }
            Message::NewMessage { author_addr, bytes } =>
            {
                if let Some(author) = clients.get_mut(&author_addr)
                {
                    if let Ok(_text) = str::from_utf8(&bytes)
                    {
                        println!("INFO: Client {author_addr} sent the message {bytes:?}");
                        for (addr, client) in clients.iter()
                        {
                            if *addr != author_addr
                            {
                                let _ = client.stream.as_ref().write(&bytes).map_err(|e| {
                                    eprintln!("ERROR: Could not broadcast message to all clients from author {author_addr}: {e}")
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

fn client(stream: Arc<TcpStream>, message: Sender<Message>) -> Result<(), ()> {
    Ok(())
}
