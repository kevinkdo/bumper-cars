extern crate websocket;

use std::collections::HashMap;
use std::str;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use websocket::{Server, Message, Sender, Receiver};
use websocket::message::Type;
use websocket::ws::dataframe::DataFrame;

static GLOBAL_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;

fn main() {
    GLOBAL_COUNTER.fetch_add(1, Ordering::SeqCst);
    let server = Server::bind("0.0.0.0:8000").unwrap();

    let txs_by_id = Arc::new(Mutex::new(HashMap::new()));

    for connection in server {
        let id = GLOBAL_COUNTER.fetch_add(1, Ordering::SeqCst);
        let request = connection.unwrap().read_request().unwrap();
        let response = request.accept();
        let client = response.send().unwrap();
        let (mut sender, mut receiver) = client.split();
        let ip = sender
            .get_mut()
            .peer_addr()
            .unwrap();
        println!("Connection from {}", ip);

        let (tx, rx) = mpsc::channel::<Message>();
        let tx = Arc::new(Mutex::new(tx));

        let tx_to_push = tx.clone();
        let txs_copy1 = txs_by_id.clone();
        let txs_copy2 = txs_by_id.clone();

        let mut txs_by_id = txs_by_id.lock().unwrap();
        txs_by_id.insert(id, tx_to_push);

        // `sender` thread
        thread::spawn(move || {
            for message in rx {
                let payload = message.payload();
                let payload = str::from_utf8(payload.as_ref());
                println!("> {:?}", payload);
                sender.send_message(&message).unwrap();

                if message.opcode == Type::Close {
                    let mut txs = txs_copy1.lock().unwrap();
                    txs.remove(&id);
                    return;
                }
            }
        });

        // `receiver` thread
        thread::spawn(move || {
            for message in receiver.incoming_messages() {
                let message: Message = message.unwrap();

                match message.opcode {
                    Type::Close => {
                        let message = Message::close();
                        let tx = tx.lock().unwrap();
                        tx.send(message).unwrap();
                        println!("Client {} disconnected", ip);
                        return;
                    },
                    Type::Ping => {
                        let message = Message::pong(message.payload);
                        let tx = tx.lock().unwrap();
                        tx.send(message).unwrap();
                    }
                    _ => {
                        let raw_payload = message.payload();
                        let raw_payload = str::from_utf8(raw_payload.as_ref());
                        println!("< {:?}", raw_payload);

                        match raw_payload {
                            // Reply with unique id only to the sender
                            Ok("getid") => {
                                let id_string = id.to_string();
                                let prefix = "getid".to_string();
                                let combined_string = prefix + &id_string;
                                let message = Message::text(combined_string);
                                let tx = tx.lock().unwrap();
                                tx.send(message).unwrap()
                            },
                            // Broadcast the message if it's a valid UTF8 encoding
                            Ok(_) => {
                                let txs = txs_copy2.lock().unwrap();
                                for (_, tx) in txs.iter() {
                                    let tx = tx.lock().unwrap();
                                    tx.send(message.clone()).unwrap();
                                }
                            }
                            // Ignore the message if it's not valid UTF8
                            Err(_) => {}
                        };
                    }
                }
            }
        });
    }
}