extern crate tun_tap;
use std::{io, vec};

fn main() -> io::Result<()> {
    let nic = tun_tap::Iface::new("tun0", tun_tap::Mode::Tun).expect("failed to cr");
    let mut buf = vec![0u8;1504];
    loop {
        let nbytes = nic.recv(&mut buf)?;
        let flags = u16::from_be_bytes([buf[0], buf[1]]);
        let protol = u16::from_be_bytes([buf[2], buf[3]]);
        eprintln!("read {} bytes: {:x?}, protol: {:x}, flags: {:x}", nbytes,  &buf[0..nbytes], protol, flags);        
    }
    Ok(())
}