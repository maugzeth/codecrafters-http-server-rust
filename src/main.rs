use std::{net::TcpListener, io::{Read, Write}};

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                println!("accepted new connection");
                let mut buf: Vec<u8> = Vec::new();
                match s.read(buf.as_mut_slice()) {
                    Ok(_) =>  println!("Buffer: {:?}", buf),
                    Err(_) => panic!("failed to read connection data into buffer"),
                };
                s.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
