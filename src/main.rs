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
        ServerObj::parse_connection(stream)
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

struct ServerObj {
    address: String,
    port: String,
    listener: TcpListener
}

enum HttpResponse {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH
}

impl ServerObj{     // can extend when more options are available
    fn from_config() -> ServerObj{
        let config_vals = load_config_file();
        ServerObj{
            address: config_vals[0].clone(),
            port: config_vals[1].clone(),
            listener: TcpListener::bind(format!("{}:{}",config_vals[0],config_vals[1]))
                .expect("Needs to load IP to bind, maybe out of network scope?")
        }
    }

    fn parse_connection(mut stream: TcpStream) {
        let mut data_buffer: [u8;1024] = [0;1024];
        stream.read(&mut data_buffer).expect("Failed reading TCP?");
        let data_string = String::from_utf8_lossy(&data_buffer[..]);
        let data_request: Vec<&str> = data_string.split(" ").collect();
        let http_type = match data_request.get(0) {
            Some(e) => {
                let response_type = match e.to_uppercase().as_str() {
                    "GET" => HttpResponse::GET,
                    "POST" => HttpResponse::POST,
                    "PUT" => HttpResponse::PUT,
                    "DELETE" => HttpResponse::DELETE,
                    "PATCH" => HttpResponse::PATCH,
                    _ => HttpResponse::GET
                };
                response_type
            },
            None => HttpResponse::GET
        };

        let resource = match data_request.get(1) {
            Some(&e) => {
                match e {
                    "/" => (ResCode::Ok, e),
                    e if e.contains("..") => (ResCode::NotFound, "servercontent/404.html"),
                    e if e.contains("~") => (ResCode::NotFound, "servercontent/404.html"),
                    _ => (ResCode::Ok, "servercontent/index.html")
                }
            },
            None => (ResCode::NotFound, "servercontent/404.html")
        };
        // To-do: Expand parse connection to allow for multiple types of HTTP requests
    }
}

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