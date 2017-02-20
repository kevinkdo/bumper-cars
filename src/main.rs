extern crate websocket;

use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc;
use websocket::{Server, Message, Sender, Receiver};
use websocket::message::Type;

fn main() {
    let server = Server::bind("0.0.0.0:8000").unwrap();

    let txs = Arc::new(Mutex::new(Vec::new()));

    for connection in server {
        let request = connection.unwrap().read_request().unwrap();
        let response = request.accept();
        let client = response.send().unwrap();
        let (mut sender, mut receiver) = client.split();
        let ip = sender
            .get_mut()
            .peer_addr()
            .unwrap();
        println!("Connection from {}", ip);

        let (tx, rx) = mpsc::channel();
        let tx = Arc::new(Mutex::new(tx));

        let tx_to_push = tx.clone();
        let txs_for_thread = txs.clone();

        let mut txs = txs.lock().unwrap();
        txs.push(tx_to_push);

        // `sender` thread
        thread::spawn(move || {
            for message in rx {
                sender.send_message(&message).unwrap();
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
                        let txs = txs_for_thread.lock().unwrap();
                        for tx in txs.iter() {
                            let tx = tx.lock().unwrap();
                            tx.send(message.clone()).unwrap();
                        }
                    }
                }
            }
        });
    }
}