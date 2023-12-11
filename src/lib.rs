use std::collections::{HashMap, VecDeque};
use std::{io, default};
use std::io::prelude::*;
use std::net::Ipv4Addr;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

mod tcp;

const SENDQUEUE_SIZE:usize = 1024;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct Quad {
    pub src: (Ipv4Addr, u16),
    pub dst: (Ipv4Addr, u16),
}

#[derive(Default)]
struct Foobar {

}

type InterfaceHandle = Arc<Foobar>;

pub struct Interface {

}

impl Drop for Interface {
    fn drop(&mut self) {
        
    }
}

#[derive(Default)]
struct ConnectionManager {

}

fn packet_loop() {
    
}

impl Interface {
    pub fn new() {
        
    }
    pub fn bind() {
        
    }
}

pub struct TcpListener {

}

impl TcpListener {
    fn accept() {
        
    }

}

pub struct TcpStream {

}

impl Drop for TcpStream {
    fn drop(&mut self) {
        
    }
}

impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Ok(0)
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(0)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl TcpStream {
    pub fn shutdown() {}
}