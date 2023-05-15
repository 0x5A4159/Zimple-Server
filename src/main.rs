use std::io::{Read, BufReader, BufRead};
use std::fs::File;
use std::fs;
use std::net::TcpListener;
use std::net::TcpStream;
use std::io::prelude::*;

fn main() {
    let server = ServerObj::from_config();
    for stream in server.listener.incoming(){
        let stream = stream.expect("TCP Listener needs to establish connection");
        parse_connection(stream)
    }
}

fn load_config_file() -> Vec<String> {
    let config_file = File::open("server/config.cfg")
        .expect("This should be a file that's always accessible");

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

    fn parse_connection(mut stream: TcpStream) {
        let mut data_buffer: [u8;1024] = [0;1024];
        stream.read(&mut data_buffer).unwrap();
        let data_string = String::from_utf8_lossy(&data_buffer[..]);
        let data_request: Vec<&str> = data_string.split(" ").collect();
        // originally i was considering just getting a slice of the first 3 bytes of the buffer
        // since all of the response headers are distinct, but if i'm going to split by space anyway
        // for the resource information this should be about as efficient.

        let http_type = match data_request.get(0) {
            Some(e) => {
                match e.to_uppercase().as_str() {
                    "GET" => HttpResponse::GET,
                    "POST" => HttpResponse::POST,
                    "PUT" => HttpResponse::PUT,
                    "DELETE" => HttpResponse::DELETE,
                    "PATCH" => HttpResponse::PATCH,
                    _ => HttpResponse::GET // just in case all else fails this should return 404 error
                }
            },
            None => HttpResponse::GET
        };

        let resource = match data_request.get(1) {
            Some(&e) => {
                match e {
                    "/" => "./server/server_content/index.html".to_string(),
                    e if e.contains("..") => "./server/server_content/404.html".to_string(),
                    e if e.contains("~") => "./server/server_content/404.html".to_string(),
                    _ => format!("./server/server_content{}", e)
                }
            }
            None => "./server/server_content/404.html".to_string()
        };

        let response_info = http_type.collect_information(resource);

        let mut raw_http_response = Vec::from(format!("HTTP/1.1 {:?}\r\nContent-Length: {}\r\n\r\n",
                                                      &response_info.response,
                                                      &response_info.length));

        raw_http_response.extend(&response_info.content);

        stream.write(&raw_http_response).unwrap();
        stream.flush().unwrap();
    }


struct ServerObj {
    address: String,
    port: String,
    listener: TcpListener
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
    fn collect_information(self, resource: String) -> GetResponse {
        match self {
            HttpResponse::GET => {
                let file_bytes = read_file_bytes(resource);
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
                .expect("Needs to load IP to bind, maybe out of network scope?")
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

fn read_file_string(file_path: String) -> String {
    let data = match fs::read_to_string(file_path){
        Ok(e) => e,
        Err(_) => "No string content at file\n\rTry Again".to_string()
    };

    data
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