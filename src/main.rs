use clap::Parser;
use itertools::Itertools;
use std::{net::TcpListener, io::{Read, Write}, collections::HashMap, thread, fs};

const BUFFER_SIZE: usize = 1024;

const SECTION_END_SEQ: &'static str = "\r\n\r\n";
const NEW_LINE_SEQ: &'static str = "\r\n";

const OK_200: &'static str = "HTTP/1.1 200 OK\r\n";
const OK_201: &'static str = "HTTP/1.1 201 OK\r\n";
const NOT_FOUND_404: &'static str = "HTTP/1.1 404 NOT FOUND\r\n";

const CONTENT_TYPE_PLAIN: &'static str = "Content-Type: text/plain\r\n";
const CONTENT_TYPE_OCTET: &'static str = "Content-Type: application/octet-stream\r\n";

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    directory: Option<String>,
}

enum HttpMethod {
    GET,
    POST
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                thread::spawn(move || {
                    let mut buf = [0; BUFFER_SIZE];
                    let response = match s.read(&mut buf) {
                        Ok(_) =>  handle_incoming_request(&buf).expect("failed to handle incoming request"),
                        Err(_) => panic!("failed to read connection data into buffer"),
                    };
                    s.write(response.as_bytes()).unwrap();
                });
            }
            Err(_) => {
                panic!("failed to read incoming tcp stream");
            }
        }
    }

    Ok(())
}

/// Incoming request handler for client.
fn handle_incoming_request(buf: &[u8]) -> Result<String, String> {
    let args = Args::parse();
    let headers = parse_headers(&buf);
    let headers_map = headers_to_map(&headers);
    let path = extract_path(&headers[0]);

    let mut response = {
        if path == "/" {
            OK_200.to_string()
        } else if path.starts_with("/echo") {
            let echo_str = path.strip_prefix("/echo/").unwrap();
            response_with_data(OK_200, CONTENT_TYPE_PLAIN, &echo_str)
        } else if path.starts_with("/user-agent") {
            let user_agent = headers_map.get("User-Agent").expect("expected User-Agent header to be present");
            response_with_data(OK_200, CONTENT_TYPE_PLAIN, &user_agent)
        } else if path.starts_with("/files") {
            let file_name = path.split("/").last().expect("no file name passed in path");
            let file_dir = args.directory.expect("no file directory provided");
            let file_path = format!("{}{}", file_dir, file_name);
            let http_method = get_http_method(&headers[0]);

            match http_method {
                HttpMethod::GET => match fs::read_to_string(&file_path) {
                    Ok(data) => response_with_data(OK_200, CONTENT_TYPE_OCTET, &data),
                    Err(_) => NOT_FOUND_404.to_string()
                },
                HttpMethod::POST => match fs::write(&file_path, parse_body(&buf)) {
                    Ok(_) =>  OK_201.to_string(),
                    Err(_) => NOT_FOUND_404.to_string()
                }
            }
        } else {
            NOT_FOUND_404.to_string()
        }
    };

    // Add EOL equivelant for response
    response.push_str("\r\n");

    Ok(response)
}

/// Parses incoming request buffer bytes into line of UTF-8 encoded strings.
fn parse_headers(buffer: &[u8]) -> Vec<String> {
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
            panic!("failed to parse lines buffer");
        }
    }
}

/// Parses incoming request body from buffer.
fn parse_body(buffer: &[u8]) -> String {
    match String::from_utf8(buffer.to_vec()) {
        Ok(raw_string) => {
            let body = raw_string.split("\r\n\r\n").collect_vec()[1];
            body.trim_matches('\0').to_string()
        },
        Err(_) => {
            panic!("failed to parse lines buffer");
        }
    }
}

/// Extracts path from incoming request.
fn extract_path(line: &String) -> String {
    let split_line: Vec<String> = line.split(" ").map(|s| s.to_string()).collect();
    split_line.get(1).expect("Expected 3 elements in split line").clone()
}

/// Append data to response.
fn response_with_data(response: &str, content_type: &str, data: &str) -> String {
    let mut response_with_content = String::from(response);
    response_with_content.push_str(content_type);
    response_with_content.push_str(format!("Content-Length: {}{}", data.as_bytes().len(), SECTION_END_SEQ).as_str());
    response_with_content.push_str(format!("{}{}", data, NEW_LINE_SEQ).as_str());
    response_with_content
}

/// Converts raw header strings into hash map.
fn headers_to_map(headers: &Vec<String>) -> HashMap<String, String> {
    let mut header_map: HashMap<String, String> = HashMap::new();
    headers.iter().for_each(|h| {
        match h.split_once(":") {
            Some(s) => header_map.insert(s.0.to_string(), s.1.trim().to_string()),
            None => header_map.insert(h.clone(), String::new())
        };
    });
    header_map
}

fn get_http_method(header: &String) -> HttpMethod {
    match header.split(" ").collect_vec()[0] {
        "POST" => HttpMethod::POST,
        "GET" => HttpMethod::GET,
        _ => panic!("unsupported http method used")
    }
}
