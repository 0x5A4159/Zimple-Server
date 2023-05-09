use std::io::{Read, BufReader, BufRead};
use std::fs::File;
use std::fs;

fn main() {
    let config_values = load_config_file();
    let server_ip = &config_values[0];          // There's likely a more elegant solution
    let server_port = &config_values[1];        // that I'm missing.
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