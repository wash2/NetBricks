use e2d2::headers::*;
use e2d2::interface::*;
use e2d2::queues::*;
use e2d2::scheduler::Executable;
use eui48::MacAddress;
use std::net::Ipv4Addr;
use std::str::FromStr;

pub struct PacketCreator {
    mac: MacHeader,
    ip: IpHeader,
    producer: MpscProducer,
}

impl<'a> PacketCreator {
    pub fn new(producer: MpscProducer) -> PacketCreator {
        let mut mac = MacHeader::new();
        mac.dst = MacAddress::new([0x68, 0x05, 0xca, 0x00, 0x00, 0xac]);
        mac.src = MacAddress::new([0x68, 0x05, 0xca, 0x00, 0x00, 0x01]);
        mac.set_etype(0x0800);
        let mut ip = IpHeader::new();
        ip.set_src(u32::from(Ipv4Addr::from_str("10.0.0.1").unwrap()));
        ip.set_dst(u32::from(Ipv4Addr::from_str("10.0.0.5").unwrap()));
        ip.set_ttl(128);
        ip.set_version(4);
        ip.set_ihl(5);
        ip.set_length(20);
        PacketCreator {
            mac: mac,
            ip: ip,
            producer: producer,
        }
    }

    #[inline]
    fn initialize_packet(&self, mut pkt: Pdu<'a>) -> Pdu<'a> {
        pkt.push_header(&self.mac);
        pkt.push_header(&self.ip);
        pkt
    }

    #[inline]
    pub fn create_packet(&self) -> Pdu {
        self.initialize_packet(Pdu::new_pdu().unwrap())
    }
}

impl Executable for PacketCreator {
    fn execute(&mut self) -> (u32, i32) {
        for _ in 0..16 {
            self.producer.enqueue_one(self.create_packet());
        }
        (16, 0)
    }
}
