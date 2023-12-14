use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::Ipv4Addr;
use std::os::unix::thread;
use std::{clone, io, vec};

use tcp::Connection;
use tcp_rust::Interface;
mod tcp;

fn main() -> io::Result<()> {
    let mut i = tcp_rust::Interface::new()?;
    eprintln!("created interface");
    let mut listener = i.bind(80)?;
    while let Ok(mut stream) = listener.accept() {
        eprintln!("got connection");
        std::thread::spawn(move || {
            stream.write(b"hello from tcp-rust!\n").unwrap();
            stream.shutdown().unwrap();
            loop {
                let mut buf = [0; 512];
                let n = stream.read(&mut buf[..]).unwrap();
                eprintln!("read {} bytes of data", n);
                if n == 0 {
                    eprintln!("no more data!");
                } else {
                    println!("{}", std::str::from_utf8(&buf[..n]).unwrap());
                }
            }
        });
    }
    Ok(())
}
