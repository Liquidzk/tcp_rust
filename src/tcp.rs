pub struct State {}

impl Default for State {
    fn default() -> Self {
        State {}
    }
}
impl State {
    pub fn on_packet<'a>(
        &mut self,
        ip_header: etherparse::Ipv4HeaderSlice<'a>,
        tcp_header: etherparse::TcpHeaderSlice<'a>,
        data: &'a [u8],
    ) {
        eprintln!(
            "From {:?}:{:?}, len:{:?}, dst:{:?}:{:?}, data:{:x?}",
            ip_header.source_addr(),
            tcp_header.source_port(),
            tcp_header.slice().len(),
            ip_header.source_addr(),
            tcp_header.destination_port(),
            data
        );
    }
}
