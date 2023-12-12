use std::collections::{HashMap, VecDeque};
use std::f32::consts::E;
use std::io::prelude::*;
use std::net::Ipv4Addr;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::{default, io};

use tcp::Connection;

mod tcp;

const SENDQUEUE_SIZE: usize = 1024;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct Quad {
    pub src: (Ipv4Addr, u16),
    pub dst: (Ipv4Addr, u16),
}

#[derive(Default)]
struct Foobar {
    manager_mutex: Mutex<ConnectionManager>,
    // 通知pending等待线程
    pending_var: Condvar,
    // 通知接收的等待线程
    rcv_var: Condvar,
}

type InterfaceHandle = Arc<Foobar>;

pub struct Interface {
    // 接口管理
    interfacehandle: Option<InterfaceHandle>,
    // 线程管理
    threadhandle: Option<thread::JoinHandle<io::Result<()>>>,
}

impl Drop for Interface {
    fn drop(&mut self) {
        self.interfacehandle
            .as_mut()
            .unwrap()
            .manager_mutex
            .lock()
            .unwrap()
            .terminate = true;
        drop(self.interfacehandle.take());

        let i = self
            .threadhandle
            .take()
            .expect("take more than once")
            .join()
            .unwrap();
    }
}

#[derive(Default)]
struct ConnectionManager {
    terminate: bool,
    connections: HashMap<Quad, Connection>,
    pending: HashMap<u16, VecDeque<Quad>>,
}

fn packet_loop(mut nic: tun_tap::Iface, interfacehandle: InterfaceHandle) -> io::Result<()> {
    let mut buf = [0u8; 1504];

    loop {}
}

impl Interface {
    pub fn new() -> io::Result<Self> {
        let nic =
            tun_tap::Iface::without_packet_info("tun0", tun_tap::Mode::Tun).expect("failed to cr");
        let ih: InterfaceHandle = Arc::default();
        let jh = {
            let ih = ih.clone();
            thread::spawn(move || packet_loop(nic, ih))
        };

        Ok(Interface {
            interfacehandle: Some(ih),
            threadhandle: Some(jh),
        })
    }
    pub fn bind(&mut self, port: u16) -> io::Result<TcpListener> {
        use std::collections::hash_map::Entry;
        let mut cm = self
            .interfacehandle
            .as_mut()
            .unwrap()
            .manager_mutex
            .lock()
            .unwrap();
        match cm.pending.entry(port) {
            Entry::Vacant(v) => {
                v.insert(VecDeque::new());
            }
            Entry::Occupied(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AddrInUse,
                    "port already bound",
                ));
            }
        }
        drop(cm);
        Ok(TcpListener {
            port: port,
            interfacehandle: self.interfacehandle.as_mut().unwrap().clone(),
        })
    }
}

//监听某端口
pub struct TcpListener {
    port: u16,
    // 应该是用来查找Connections里是否存在的
    interfacehandle: InterfaceHandle,
}

impl TcpListener {
    fn accept(&mut self) -> io::Result<TcpStream> {
        let mut cm = self.interfacehandle.manager_mutex.lock().unwrap();
        loop {
            if let Some(quad) = cm
                .pending
                .get_mut(&self.port)
                .expect("this port is not in listening.")
                .pop_front()
            {
                return Ok(TcpStream {
                    quad: quad,
                    interfacehandlle: self.interfacehandle.clone(),
                });
            }
            cm = self.interfacehandle.pending_var.wait(cm).unwrap();
        }
    }
}

pub struct TcpStream {
    quad: Quad,
    interfacehandlle: InterfaceHandle,
}

impl Drop for TcpStream {
    fn drop(&mut self) {
        let mut cm = self.interfacehandlle.manager_mutex.lock().unwrap();
        // TODO: 先发FIN过去，然后把quad drop掉
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
