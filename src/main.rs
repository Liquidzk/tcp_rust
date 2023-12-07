extern crate tun_tap;
use std::{io, vec};

fn main() -> io::Result<()> {
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
            Ok(p) => {
                let src = p.source_addr();
                let dst = p.destination_addr();
                let ip_protol = p.protocol();
                let payload = p.payload_len();
                if ip_protol != 0x06 {
                    //忽略除了TCP以外的报文
                    eprintln!(
                        "Can't parse if it is not a TCP packet. IP_Protol: {:x}",
                        ip_protol
                    );
                    continue;
                }
                match etherparse::TcpHeaderSlice::from_slice(&buf[4 + p.slice().len()..]) {
                    Ok(t) => {
                        eprintln!(
                            "From {:?}:{:?}, len:{:?}, dst:{:?}:{:?}",
                            src,
                            t.source_port(),
                            t.slice().len(),
                            dst,
                            t.destination_port()
                        );
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
