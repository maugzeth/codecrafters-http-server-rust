use std::{net::TcpListener, io::{Read, Write}};

const BUFFER_SIZE: usize = 1024;

const OK_200: &'static str = "HTTP/1.1 200 OK\r\n\r\n";
const NOT_FOUND_404: &'static str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                println!("Accepted new connection");

                let mut buf = [0; BUFFER_SIZE];
                let response = match s.read(&mut buf) {
                    Ok(_) =>  handle_incoming_request(&buf).expect("failed to handle incoming request"),
                    Err(_) => panic!("failed to read connection data into buffer"),
                };

                s.write(response.as_bytes()).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_incoming_request(buf: &[u8]) -> Result<String, String> {
    let lines = parse_lines(&buf);
    println!("Buffer: {:?}", lines);
    let path = extract_path(&lines[0]);
    println!("Path: {:?}", path);
    let response = match_path_to_response(&path);
    println!("Response: {:?}", response);
    Ok(response)
}

fn parse_lines(buffer: &[u8]) -> Vec<String> {
    match String::from_utf8(buffer.to_vec()) {
        Ok(raw_string) => {
            let mut lines: Vec<String> = raw_string
                .split("\r\n")
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .collect();
            // Remove the last line which is just padding of zeros from buffer
            lines.pop();
            return lines;
        },
        Err(_) => {
            panic!("Failed to parse lines buffer");
        }
    }
}

fn extract_path(line: &String) -> String {
    let split_line: Vec<String> = line.split(" ").map(|s| s.to_string()).collect();
    split_line.get(1).expect("Expected 3 elements in split line").clone()
}

fn match_path_to_response(path: &str) -> String {
    match path {
        "/" => OK_200.to_string(),
        _ => NOT_FOUND_404.to_string(),
    }
}