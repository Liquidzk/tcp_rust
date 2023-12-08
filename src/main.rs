extern crate tun_tap;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::{clone, io, vec};
mod tcp;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
struct Quad {
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}

fn main() -> io::Result<()> {
    let mut connections: HashMap<Quad, tcp::State> = HashMap::new();
    let nic = tun_tap::Iface::new("tun0", tun_tap::Mode::Tun).expect("failed to cr");
    let mut buf = vec![0u8; 1504];
    loop {
        let nbytes = nic.recv(&mut buf)?;
        let flags = u16::from_be_bytes([buf[0], buf[1]]);
        let protol = u16::from_be_bytes([buf[2], buf[3]]);
        if protol != 0x0800 {
            //先忽略除了IPv4报文之外的报文
            eprintln!(
                "Can't parse if it is not a IPv4 packet. Protol: {:x} ",
                protol
            );
            continue;
        }

        match etherparse::Ipv4HeaderSlice::from_slice(&buf[4..]) {
            Ok(ip_header) => {
                let src = ip_header.source_addr();
                let dst = ip_header.destination_addr();
                let ip_protol = ip_header.protocol();
                let payload = ip_header.payload_len();
                if ip_protol != 0x06 {
                    //忽略除了TCP以外的报文
                    eprintln!(
                        "Can't parse if it is not a TCP packet. IP_Protol: {:x}",
                        ip_protol
                    );
                    continue;
                }
                match etherparse::TcpHeaderSlice::from_slice(&buf[4 + ip_header.slice().len()..]) {
                    Ok(tcp_header) => {
                        let data_start = 4 + ip_header.slice().len() + tcp_header.slice().len();
                        connections
                            .entry(Quad {
                                src: (src, tcp_header.source_port()),
                                dst: (dst, tcp_header.destination_port()),
                            })
                            .or_default().on_packet(ip_header, tcp_header, &buf[data_start..nbytes]);
                        
                    }
                    Err(e) => {
                        eprintln!("TCP parse error: {:?}", e);
                    }
                }
                // eprintln!(
                //     "read {} bytes, protol: {:x}, source:{:?}, dst:{:?}, payload:{:?} ",
                //     nbytes,
                //     ip_protol,
                //     src,
                //     dst,
                //     payload
                // );
            }
            Err(e) => {
                eprintln!("ignoring weird packet {:?}", e);
            }
        }
    }
    Ok(())
}
