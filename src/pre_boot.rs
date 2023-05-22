use std::fs;
use std::thread;

const DIR_NAMES: [&str;2] = ["./server", "./server/server_content"];
const FILE_NAMES: [(&str, &str);5] = [  // static size for const, could include private fn main to run this as dynamically sized
    ("./server/server_content/index.html", "<!DOCTYPE html> <html lang=\"en\"> <head> <style> body { background-color: whitesmoke; } .Main { -ms-overflow-style: none; scrollbar-width: none; margin-top:0; padding-top:0; position:fixed; overflow-y: scroll; height:100%; width:66%; top: 50%; left: 50%; transform:translate(-50%,-50%); padding-left:20px; padding-right:20px; background-color:dimgray; } .Main::-webkit-scrollbar{ display:none; } .Main #Header { margin-top:0.2em; font-size: 100px; width:4em; color: whitesmoke; background-color: #234f4b; position: relative; text-align: center; left:50%; transform: translate(-50%, 0); border-radius: 10px; padding:10px; text-shadow: 0px 6px black; font-family: 'DejaVu Sans Mono'; } p { font-size: 25px; color: whitesmoke; text-shadow: 0px 2px black; font-family: 'DejaVu Sans Mono'; } </style> </head> <body> <div class=\"Main\"> <h1 id=\"Header\">Zimple</h1> <p>Zimple is a simple light-weight HTTP server, primarily dedicated towards localhost traffic.</p> <p> Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum. </p> </div> </body> </html>"),
    ("./server/server_content/404.html","<h1>ERROR 404</h1><p>Not Found</p>"),
    ("./server/server_content/favicon.ico",""),
    ("./server/config.cfg","ServerIP=localhost\nServerPort=8080\nHeaderSize=1024\nPageLoadCounter=F"),
    ("./server/stats.json","{\"url\" : \"./server/server_content/index.html\", \"count\" : 0}")
];

fn file_check(file_name: &str, content: &str) {
    if !(std::path::Path::new(file_name).exists()) {
        fs::write(file_name, content)
            .expect("No file permissions to write");
        println!("+ File {file_name} created!")
    }
}

fn dir_check(dir_name: &str) {
    if !(std::path::Path::new(dir_name).is_dir()) {
        fs::create_dir(dir_name)
            .expect("No file permissions to make directory");
        println!("+ Directory {dir_name} created")
    }
}

pub fn preload() {
    println!("Beginning setup, checking for files");
    for dir in DIR_NAMES {
        dir_check(dir);
    }

    // directories need to be in place first before files can be handled so i'm purposefully excluding directories from thread pool.

    let mut thread_handles = vec![];

    for file in FILE_NAMES {
        let handle = thread::spawn(move || {file_check(file.0,file.1);});
        thread_handles.push(handle);
    }

    for thread in thread_handles {
        thread.join().unwrap();
    }

    println!("Config files exist\nCan begin run server attempt.");
}