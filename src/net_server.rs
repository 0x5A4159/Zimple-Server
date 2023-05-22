use std::io::{Read, BufReader, BufRead};
use std::fs::File;
use std::net::TcpListener;
use std::net::TcpStream;
use std::io::prelude::*;
mod page_counter;

pub fn start_server() {
    let server = ServerObj::from_config();
        println!("Server successfully loaded config and bound to {}:{}", &server.address, &server.port);

    for stream in server.listener.incoming(){
        let stream = stream.expect("TCP Listener needs to establish connection");
        parse_connection(stream, &server)
    }
}


fn parse_connection(mut stream: TcpStream,server_info: &ServerObj) {
    let header_bytes = stream_reader(&stream,3); // request type (first 3 letters)

    let http_type = match header_bytes[..] {
        [71,69,84] => HttpResponse::GET,
        [80,79,83] => HttpResponse::POST,
        [80,85,84] => HttpResponse::PUT,    // UTF8 vectors of the first 3 letters of http protocols
        [68,69,76] => HttpResponse::DELETE,
        [80,65,84] => HttpResponse::PATCH,
        _ => HttpResponse::GET
    };

    let response_info = http_type.collect_information(&mut stream, server_info);

    let mut raw_http_response = Vec::from(format!("HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n",
                                                  &response_info.response.make(),
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
        .to_string())
        .collect()
}

struct ServerObj {
    address: String,
    port: String,
    listener: TcpListener,
    header_size: usize,
    load_counter: bool
}

#[derive(PartialEq)] // need this to compare enum for impl
enum HttpResponse {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH
}


#[derive(Debug)]
struct GetResponse {
    response: ResCode,
    length: usize,
    content: Vec<u8>
}

fn simple_find (resource: &std::borrow::Cow<str>, to_find: &str) -> String {
    let mut return_obj = String::new();     // This is a low effort finder to quickly return a line
    for chunk in resource.split("\n") { // that contains some value we're looking for
        if chunk.to_lowercase().contains(to_find){
            return_obj = chunk.to_string().to_lowercase();
            break;      // there's probably a much more efficient way to do this.
        }
    }
    return_obj
}

impl HttpResponse {     // trying to break up the logic chunks here so it isn't as cluttered
    fn collect_information(self, stream: &mut TcpStream,server_info: &ServerObj) -> GetResponse {
        match self {
            HttpResponse::GET => {
                let data_buffer = stream_reader(&stream,server_info.header_size);
                let resource = String::from_utf8_lossy(&data_buffer[..]);

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

                if server_info.load_counter {
                    // todo: add counter functionality
                    // potentially doing so by getting the resource name and storing as json to the file, can use
                    // serde to serialize and deserialize it for math operations.
                    page_counter::incr_count(&resource_path);
                }

                let file_bytes = read_file_bytes(resource_path);
                file_bytes
            }
            HttpResponse::POST => {
                let mut data_buffer = stream_reader(&stream,server_info.header_size); // just capturing 1024 bytes to hopefully catch content-size
                let resource = String::from_utf8_lossy(&mut data_buffer[..]); // read bytes as string

                let content_length = simple_find(&resource,"content-length:");

                let accept_type = simple_find(&resource,"accept:"); // can do simple matching based on something like
                                                                                 // match { "accept:application/json" => {}. "accept:text/html" => {} } etc.
                let content_type = simple_find(&resource, "content-type:");

                let content_length_value = content_length.split(":")
                    .collect::<Vec<_>>()
                    .get(1)     // should return right-hand side of content_length to give us string of content length declaration
                    .unwrap_or(&"0") // default should anything go wrong, no extensibility
                    .parse()
                    .unwrap_or(0); // another default should anything go wrong somehow.

                data_buffer.extend(stream_reader(&stream,content_length_value)); // reads the remaining size of the request and extends the data buffer
                // original 1024 array: [a,b,c...] + remaining content length = [a,b,c...x,y,z,0,0] (trailing zero should be fine)
                // This should be a robust enough solution for variable content length, and the initial 1024 byte array should capture
                // a good portion of headers, at least enough to get content length to extend array

                let resource = String::from_utf8_lossy(&data_buffer[..]); // recreate string array to include extended byte array

                let body_content: Vec<&str> = resource.split("\r\n\r\n").collect();

                let body_content = match body_content.get(1){
                    Some(e) => e.to_string(),
                    None => "".to_string()
                };  // gets the body of the post by getting the right-hand split of the double linebreak in requests.

                let context = match content_type.replace("\n","").as_str() {
                    "content-type: application/x-www-form-urlencoded" => {
                        // todo
                    },
                    "content-type: application/json" => {
                        // todo
                    },
                    "content-type: application/xml" => {
                        // todo
                    },
                    "content-type: text/html; charset=utf-8" => {
                        // todo
                    }
                    _ => {
                        // default to content-type: text/html
                    }
                };

                let mut final_response = "".to_string(); // this can be dictated by accept type and data fed to it, be it json or txt or whatever

                GetResponse{        // place holder response
                    response:ResCode::Ok,
                    length: 200_usize,
                    content: Vec::from(final_response)
                }

            },
            _ => {
                GetResponse{
                    response:ResCode::NotImplemented,
                    length: 40_usize,
                    content: Vec::from("<h1>501 Error</h1><p>Not Implemented</p>")
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
            header_size: config_vals[2].clone().parse().expect("Config value header_size should be numerical"),
            load_counter: match config_vals[3].clone().as_str() {
                "F" => false,
                "T" => true,
                _ => false
            }
        }
    }
}

#[derive(Debug)]
enum ResCode {
    Ok,
    MovedPermanently,
    Unauthorized,
    Forbidden,
    NotFound,
    NotImplemented
}

impl ResCode {
    fn make(&self) -> String {
        match self {
            ResCode::Ok => {String::from("200")},
            ResCode::MovedPermanently => {String::from("301")},
            ResCode::Unauthorized => {String::from("401")},
            ResCode::Forbidden => {String::from("403")},
            ResCode::NotFound => {String::from("404")},
            ResCode::NotImplemented => {String::from("501")}
        }
    }
}

fn read_file_bytes(file_path: String) -> GetResponse {
    let mut found: bool = true;

    let file: File = {File::open(file_path)
        .unwrap_or_else(|_| {
            found = false;
            File::open("./server/server_content/404.html").unwrap()
        })};

    let page_found: ResCode = match found {
        true => {
            ResCode::Ok
        }
        false => {
            ResCode::NotFound
        }
    };
    let mut buffer = Vec::new();
    let mut reader = BufReader::new(file);

    reader.read_to_end(&mut buffer).unwrap();
    GetResponse {
        response: page_found,
        length: buffer.len(),
        content: buffer
    }
}