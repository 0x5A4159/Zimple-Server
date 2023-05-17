use std::io::{Read, BufReader, BufRead};
use std::fs::File;
use std::net::TcpListener;
use std::net::TcpStream;
use std::io::prelude::*;

fn main() {
    let server = ServerObj::from_config();
    println!("Server successfully loaded config and bound to {}:{}", &server.address, &server.port);

    for stream in server.listener.incoming(){
        let stream = stream.expect("TCP Listener needs to establish connection");
        parse_connection(stream, &server)
    }
}

fn parse_connection(mut stream: TcpStream,server_info: &ServerObj) {
    let header_bytes = stream_reader(&stream,3); // request type (first 3 letters)
    let data_buffer = stream_reader(&stream,server_info.header_size);
    // header_size set in config, maybe need elastic approach for POST/PUT ?

    let data_string = String::from_utf8_lossy(&data_buffer[..]);

    let http_type = match header_bytes[..] {
        [71,69,84] => HttpResponse::GET,
        [80,79,83] => HttpResponse::POST,
        [80,85,84] => HttpResponse::PUT,    // UTF8 vectors of the first 3 letters of http protocols
        [68,69,76] => HttpResponse::DELETE,
        [80,65,84] => HttpResponse::PATCH,
        _ => HttpResponse::GET
    };

    let response_info = http_type.collect_information(data_string);

    let mut raw_http_response = Vec::from(format!("HTTP/1.1 {:?}\r\nContent-Length: {}\r\n\r\n",
                                                  &response_info.response,
                                                  &response_info.length));

    raw_http_response.extend(&response_info.content);

    stream.write(&raw_http_response).unwrap();
    stream.flush().unwrap();
}

fn stream_reader(mut stream: &TcpStream, size: usize) -> Vec<u8> {
    let mut return_bytes: Vec<u8> = vec![0;size]; // very cool of native vectors to not allow non constant size
    stream.read(&mut return_bytes).unwrap();
    return_bytes
}

fn load_config_file() -> Vec<String> {
    let config_file = File::open("./server/config.cfg")
        .expect("./server/config.cfg is missing or inaccessible");

    let reader = BufReader::new(config_file);

    reader.lines().map(|l| l        // Iterating over config file and stripping left-side
        .unwrap()                         // storing right side to vector for server struct
        .to_string()
        .split("=")
        .collect::<Vec<&str>>()
        .get(1)
        .unwrap()
        .to_string()
    ).collect()
}

struct ServerObj {
    address: String,
    port: String,
    listener: TcpListener,
    header_size: usize
}

#[derive(PartialEq)] // need this to compare enum for impl
enum HttpResponse {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH
}

struct GetResponse {
    response: ResCode,
    length: usize,
    content: Vec<u8>
}

impl HttpResponse {     // trying to break up the logic chunks here so it isn't as cluttered
    fn collect_information(self, resource: std::borrow::Cow<str>) -> GetResponse {
        match self {
            HttpResponse::GET => {
                let data_request: Vec<&str> = resource.split(" ").collect();

                let resource_path = match data_request.get(1) {
                    Some(&e) => {
                        match e {
                            "/" => "./server/server_content/index.html".to_string(),
                            e if e.contains("..") => "./server/server_content/404.html".to_string(),
                            e if e.contains("~") => "./server/server_content/404.html".to_string(),
                            _ => format!("./server/server_content{}", e)
                        }
                    }
                    None => "./server/server_content/404.html".to_string()
                };  // may need tokenization for fields

                let file_bytes = read_file_bytes(resource_path);
                GetResponse{
                    response: ResCode::Ok,
                    length: file_bytes.len(),
                    content: file_bytes
                }
            }
            _ => {
                GetResponse{
                    response:ResCode::NotFound,
                    length: 200_usize,
                    content: Vec::from("<h1>404 Error</h1><p>Not Found</p>")
                }
                // place holder so i can test GET functionality, will need to implement functions for all response types
            }
        }
    }
}

impl ServerObj {
    // can extend when more options are available
    fn from_config() -> ServerObj {
        let config_vals = load_config_file();
        ServerObj {
            address: config_vals[0].clone(),
            port: config_vals[1].clone(),
            listener: TcpListener::bind(format!("{}:{}", config_vals[0], config_vals[1]))
                .expect("Needs to load IP to bind, maybe out of network scope?"),
            header_size: config_vals[2].clone().parse().expect("Config value header_size should be numerical")
        }
    }
}

#[derive(Debug)]
enum ResCode {
    Ok,
    MovedPermanently,
    Unauthorized,
    Forbidden,
    NotFound
}

fn read_file_bytes(file_path: String) -> Vec<u8>{
    let file = match File::open(file_path) {
        Ok(e) => e,
        Err(_) => File::open("server/server_content/404.html")
            .expect("Default file should be in server_content for backup on bad request")
            // Err can return 404.html instead of empty default file
        };

    let mut buffer = Vec::new();
    let mut reader = BufReader::new(file);

    reader.read_to_end(&mut buffer).unwrap();

    buffer
}