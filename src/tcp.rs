pub enum State {
    Closed,
    Listen,
    SynRecv,
    Establish,
}

impl Default for State {
    fn default() -> Self {
        State::Listen
    }
}
impl State {
    pub fn on_packet<'a>(
        &mut self,
        nic: &tun_tap::Iface,
        ip_header: etherparse::Ipv4HeaderSlice<'a>,
        tcp_header: etherparse::TcpHeaderSlice<'a>,
        data: &'a [u8],
    ) -> std::io::Result<usize> {
        let mut buf = [0u8; 1500];
        let mut writter = &mut buf[..];
        match *self {
            State::Closed => {
                //throw out the packet
                return Ok(0);
            }
            State::Listen => {
                if !tcp_header.syn() {
                    return Ok(0);
                }
                //send a ack
                let mut ack_syn = etherparse::TcpHeader::new(
                    tcp_header.destination_port(),
                    tcp_header.source_port(),
                    0,
                    0,
                );
                ack_syn.syn = true;
                ack_syn.ack = true;
                let ipv4_packet = etherparse::Ipv4Header::new(
                    ack_syn.header_len(),
                    64,
                    6,
                    ip_header.destination(),
                    ip_header.source(),
                );
                ipv4_packet.write(&mut writter).unwrap();
                ack_syn.write(&mut writter).unwrap();
                let writer = writter.len();
                nic.send(&buf[..writer]).unwrap();
                return Ok(writer);
            }
            State::SynRecv => {}
            State::Establish => {}
        }
        eprintln!(
            "From {:?}:{:?}, len:{:?}, dst:{:?}:{:?}, data:{:x?}",
            ip_header.source_addr(),
            tcp_header.source_port(),
            tcp_header.slice().len(),
            ip_header.source_addr(),
            tcp_header.destination_port(),
            data
        );
        Ok(0)
    }
}
