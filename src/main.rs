extern crate websocket;

use std::sync::{Arc, Mutex};
use std::thread;
use websocket::{Server, Message, Sender, Receiver};
use websocket::message::Type;

fn main() {
    let server = Server::bind("0.0.0.0:8000").unwrap();

    let senders = Arc::new(Mutex::new(Vec::new()));

    for connection in server {
        let request = connection.unwrap().read_request().unwrap();
        let response = request.accept();

        let mut client = response.send().unwrap();
        let ip = client.get_mut_sender()
            .get_mut()
            .peer_addr()
            .unwrap();
        println!("Connection from {}", ip);
        
        let (sender, mut receiver) = client.split();
        let sender = Arc::new(Mutex::new(sender));

        let senders_clone = senders.clone();
        let sender_clone = sender.clone();

        {
            let mut senders = senders.lock().unwrap();
            senders.push(sender);
        }

        thread::spawn(move || {
            for message in receiver.incoming_messages() {
                let message: Message = message.unwrap();

                match message.opcode {
                    Type::Close => {
                        let message = Message::close();
                        let mut sender_clone = sender_clone.lock().unwrap();
                        sender_clone.send_message(&message).unwrap();
                        println!("Client {} disconnected", ip);
                        return;
                    },
                    Type::Ping => {
                        let message = Message::pong(message.payload);
                        let mut sender_clone = sender_clone.lock().unwrap();
                        sender_clone.send_message(&message).unwrap();
                    }
                    _ => {
                        let senders_clone = senders_clone.lock().unwrap();
                        for sender in senders_clone.iter() {
                            let mut sender = sender.lock().unwrap();
                            sender.send_message(&message).unwrap()
                        }
                    }
                }
            }
        });
    }
}