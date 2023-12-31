use std::{net::TcpListener, io::{Read, Write}};

//
//  CONSTANTS
//

const BUFFER_SIZE: usize = 1024;

const SECTION_END_SEQ: &'static str = "\r\n\r\n";
const NEW_LINE_SEQ: &'static str = "\r\n";

const OK_200: &'static str = "HTTP/1.1 200 OK\r\n";
const NOT_FOUND_404: &'static str = "HTTP/1.1 404 NOT FOUND\r\n";
const CONTENT_TYPE_PLAIN: &'static str = "Content-Type: text/plain\r\n";

//
//  MAIN FUNCTION
// 

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                println!("Accepted new connection");

                let mut buf = [0; BUFFER_SIZE];
                let response = match s.read(&mut buf) {
                    Ok(_) =>  handle_incoming_request(&buf)?,
                    Err(_) => panic!("failed to read connection data into buffer"),
                };

                s.write(response.as_bytes()).unwrap();
            }
            Err(_) => {
                panic!("failed to read incoming tcp stream");
            }
        }
    }

    Ok(())
}

//
//  UTILITIES
//

/// Incoming request handler for client.
fn handle_incoming_request(buf: &[u8]) -> Result<String, String> {
    let lines = parse_lines(&buf);
    println!("Buffer: {:?}", lines);
    
    let path = extract_path(&lines[0]);
    println!("Path: {:?}", path);

    let mut response = {
        if path == "/" {
            OK_200.to_string()
        } else if path.starts_with("/echo/") {
            let echo_str = path.strip_prefix("/echo/").unwrap();
            response_with_data(OK_200, &echo_str)
        } else {
            NOT_FOUND_404.to_string()
        }
    };

    // Add EOL equivelant for response
    response.push_str("\r\n");

    println!("Response: {:?}", response);
    Ok(response)
}

/// Parses incoming request buffer bytes into line of UTF-8 encoded strings.
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

/// Extracts path from incoming request.
fn extract_path(line: &String) -> String {
    let split_line: Vec<String> = line.split(" ").map(|s| s.to_string()).collect();
    split_line.get(1).expect("Expected 3 elements in split line").clone()
}

/// Append data to response.
fn response_with_data(response: &str, data: &str) -> String {
    let mut response_with_content = String::from(response);
    response_with_content.push_str(CONTENT_TYPE_PLAIN);
    response_with_content.push_str(format!("Content-Length: {}{}", data.as_bytes().len(), SECTION_END_SEQ).as_str());
    response_with_content.push_str(format!("{}{}", data, NEW_LINE_SEQ).as_str());
    response_with_content
}