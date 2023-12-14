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

    loop {
        // 这个玩意可以获取原始的文件描述符
        use std::os::unix::io::AsRawFd;
        let mut pfd = [nix::poll::PollFd::new(
            &(nic.as_raw_fd()),
            nix::poll::PollFlags::POLLIN,
        )];
        let n = nix::poll::poll(&pfd, 10).map_err(|e| e).unwrap()?;
        assert_ne!(n, -1);
        if n == 0 {
            let cm = interfacehandle.manager_mutex.lock().unwrap();
            for con in cm.connections.values_mut() {
                con.on_tick(&mut nic)?;
            }
            continue;
        }
        assert_eq!(n, 1);
        let nbytes = nic.recv(&mut buf[..])?;

        match etherparse::Ipv4HeaderSlice::from_slice(&buf[..nbytes]) {
            Ok(ipheader) => {
                let src = ipheader.source_addr();
                let dst = ipheader.destination_addr();
                if ipheader.protocol() != 0x06 {
                    eprintln!("not tcp protocol");
                    continue;
                }
                match etherparse::TcpHeaderSlice::from_slice(&buf[ipheader.slice().len()..nbytes]) {
                    Ok(tcp_header) => {
                        use std::collections::hash_map::Entry;
                    }
                    Err(e) => {
                        eprint!("tcp parse badly");
                    }
                }
            }
            Err(_) => {
                eprintln!("ignoring weird packet {:?}", e);
            }
        }
    }
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
    pub fn accept(&mut self) -> io::Result<TcpStream> {
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
        let mut cm = self.interfacehandlle.manager_mutex.lock().unwrap();
        loop {
            let c = cm.connections.get_mut(&self.quad).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "Stream was terminated unexpectedly. ",
                )
            })?;

            if c.is_rsv_closed() && c.incoming.is_empty() {
                return Ok(0);
            }

            if !c.incoming.is_empty() {
                let (head, tail) = c.incoming.as_slices();
                let hread = std::cmp::min(buf.len(), head.len());
                buf[..hread].copy_from_slice(&head[..hread]);
                c.incoming.drain(..hread);
                return Ok(hread);
            }

            cm = self.interfacehandlle.rcv_var.wait(cm).unwrap();
        }
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut cm = self.interfacehandlle.manager_mutex.lock().unwrap();
        let c = cm.connections.get_mut(&self.quad).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "stream was terminated unexpectedly",
            )
        })?;
        if c.unacked.len() >= SENDQUEUE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "too many bytes buffered",
            ));
        }
        let nwrite = std::cmp::min(buf.len(), SENDQUEUE_SIZE - c.unacked.len());
        c.unacked.extend(buf[..nwrite].iter());
        Ok(nwrite)
    }
    fn flush(&mut self) -> io::Result<()> {
        let mut cm = self.interfacehandlle.manager_mutex.lock().unwrap();
        let c = cm.connections.get_mut(&self.quad).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "stream was terminated unexpectedly",
            )
        })?;
        if c.unacked.is_empty() {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "too many bytes buffered",
            ))
        }
    }
}

impl TcpStream {
    pub fn shutdown(&mut self) -> io::Result<()> {
        let mut cm = self.interfacehandlle.manager_mutex.lock().unwrap();
        let c = cm.connections.get_mut(&self.quad).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "stream was terminated unexpectedly",
            )
        })?;

        c.close()
    }
}
