use bitflags::bitflags;
use std::collections::{BTreeMap, VecDeque};
use std::{io, time};

bitflags! {
    pub(crate) struct Avaliable: u8 {
        const READ = 0b00000001;
        const WRITE = 0b00000010;
    }
}

pub enum State {
    //Closed,
    //Listen,
    SynRecv,
    Establish,
    // 关闭阶段1
    FinWait1,
    // 关闭阶段2
    FinWait2,
    // 关闭完了，等待对方确定收到ACK
    TimeWait,
}

pub struct Timers {
    // 追踪每个发送包的平均时间
    send_times: BTreeMap<u32, time::Instant>,
    // 平滑往返时间，估计往返时间。
    //α（一个平滑因子，例如 0.125）srtt = (1 - α) * srtt + α * rtt_sample，这里 rtt_sample 是新的 RTT 测量值。
    srtt: f64,
}

pub struct Connection {
    state: State,
    send: SendSequenceSpace,
    recv: RecvSequenceSpace,
    ip_header: etherparse::Ipv4Header,
    tcp_header: etherparse::TcpHeader,
    timers: Timers,
    closed: bool,

    pub incoming: VecDeque<u8>,
    pub unacked: VecDeque<u8>,

    closed_at: Option<u32>,
}

impl Connection {
    pub fn is_rsv_closed(&self) -> bool {
        true
    }
    pub fn availablity(&self) -> bool {
        true
    }
}

// RFC 793 S3.2
pub struct SendSequenceSpace {
    // "Unacknowledged"（未确认的）。这是已发送但尚未收到确认的数据的序列号的最小值。
    una: u32,
    // nxt: "Next"（下一个）。这是下一个要发送的数据的序列号。
    nxt: u32,
    // "Window"（窗口）。这是接收方当前允许发送方发送的数据量，是一种流控制机制。
    wnd: u16,
    // "Urgent Pointer"（紧急指针）。当设置为真（true）时，表示有紧急数据需要被处理。
    up: bool,
    /*
       wl1, wl2: 这两个变量与 "Window Update"（窗口更新）相关。
       它们用于确定何时可以更新窗口的大小。
       wl1 记录了最后一次接收窗口更新的序列号，
       而 wl2 记录了最后一次接收窗口更新的确认号。
    */
    wl1: u32,
    wl2: u32,
    /**
       "Initial Send Sequence number"（初始发送序列号）。
       这是一个连接开始时的序列号，用于数据包的排序和丢失数据的检测。
    **/
    iss: u32,
}

pub struct RecvSequenceSpace {
    // 下一个期望接收的数据的序列号
    nxt: u32,
    // 接收方愿意接收的数据量
    wnd: u16,
    // 表示是否接收到紧急数据
    up: bool,
    // 连接开始时接收的第一个数据字节的序列号
    irs: u32,
}

impl Connection {
    pub fn on_packet<'a>(
        &mut self,
        nic: &tun_tap::Iface,
        ip_header: etherparse::Ipv4HeaderSlice<'a>,
        tcp_header: etherparse::TcpHeaderSlice<'a>,
        data: &'a [u8],
    ) -> std::io::Result<usize> {
        let mut buf = [0u8; 1500];
        let mut writter = &mut buf[..];
        match self.state {
            State::SynRecv => {
                //send a ack
                if tcp_header.syn() {
                    let mut ack_syn = etherparse::TcpHeader::new(
                        tcp_header.destination_port(),
                        tcp_header.source_port(),
                        self.send.iss,
                        self.send.wnd,
                    );
                    ack_syn.acknowledgment_number = tcp_header.sequence_number() + 1;
                    ack_syn.syn = true;
                    ack_syn.ack = true;
                    let ipv4_packet = etherparse::Ipv4Header::new(
                        ack_syn.header_len(),
                        64,
                        6,
                        ip_header.destination(),
                        ip_header.source(),
                    );
                    //kernel does that
                    //ack_syn.checksum = ack_syn.calc_checksum_ipv4(&ipv4_packet, &[]).expect("failed to copute checsum");

                    ipv4_packet.write(&mut writter).unwrap();
                    ack_syn.write(&mut writter).unwrap();
                    let wsize = writter.len();
                    let dbg = nic.send(&buf[..buf.len() - wsize]);
                    eprintln!("{:?}", dbg);
                    return Ok(wsize);
                } else if tcp_header.ack() {
                    self.state = State::Establish;
                    self.recv.nxt += 1;
                }

                return Ok(0);
            }
            State::Establish => {
                eprintln!("establish");
                return Ok(0);
            }
            State::FinWait1 => {}
            State::FinWait2 => {}
            State::TimeWait => {}
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

    pub fn accept<'a>(
        nic: &tun_tap::Iface,
        ip_header: etherparse::Ipv4HeaderSlice<'a>,
        tcp_header: etherparse::TcpHeaderSlice<'a>,
    ) -> Self {
        let c = Connection {
            state: State::SynRecv,
            send: SendSequenceSpace {
                iss: 0,
                una: 0,
                nxt: 1,
                wnd: 10,
                up: false,
                wl1: 0,
                wl2: 0,
            },
            recv: RecvSequenceSpace {
                nxt: tcp_header.sequence_number() + 1,
                wnd: 10,
                up: false,
                irs: tcp_header.sequence_number(),
            },
            timers: todo!(),
            closed: todo!(),
            ip_header: todo!(),
            tcp_header: todo!(),
            incoming: todo!(),
            unacked: todo!(),
            closed_at: todo!(),
        };

        c
    }
    // 当前往 nic 中写入
    pub fn write() {}
    // 发送一个rst报文重置TCP连接
    pub fn send_rst() {}
    //设计为定期处理TCP连接的状态
    pub fn on_tick() {}
    //关闭连接
    pub fn close(&mut self) -> io::Result<()> {
        self.closed = true;
        match self.state {
            State::SynRecv | State::Establish => {
                self.state = State::FinWait1;
            }
            State::FinWait1 | State::FinWait2 => {}
            State::TimeWait => {
                return Err(io::Error::new(
                    io::ErrorKind::NotConnected,
                    "all ready closed",
                ));
            }
        }
        Ok(())
    }
}

fn wrapping_lt(lhseq: u32, rhseq: u32) -> bool {
    lhseq.wrapping_sub(rhseq) > (1 << 31)
}
fn is_between_wrapped(start: u32, x: u32, end: u32) -> bool {
    wrapping_lt(start, x) && wrapping_lt(x, end)
}
