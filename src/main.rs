mod tor;

use std::io::{Read, Write};

use tor::Tor;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("{}", args[1]);

    if args.len() < 3 {
        eprintln!("Usage: {} <ip> <port>", args[0]);
        std::process::exit(1);
    }

    let mut tor = Tor::connect(&args[1], &args[2]);
    tor.stream().write_all(b"GET / HTTP/1.0\r\n\r\n")
        .expect("Failed to write to Tor stream");


    let mut buf = Vec::new();
    tor.stream().read_to_end(&mut buf).expect("Failed to read from Tor stream");
    println!("Response: {}", String::from_utf8_lossy(&buf));

}
