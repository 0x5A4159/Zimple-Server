mod net_server;
mod pre_boot;

fn main() {
    pre_boot::preload();
    println!("Starting Net Server");
    net_server::start_server();
}