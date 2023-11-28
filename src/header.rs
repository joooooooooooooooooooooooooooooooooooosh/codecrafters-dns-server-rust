use bytes::{BufMut, BytesMut};

pub struct Header {
    pub packet_id: u16,
    pub qr_indicator: QueryResponse,
    pub opcode: u8,
    pub authoritative_answer: bool,
    pub truncation: bool,
    pub recursion_desired: bool,
    pub recursion_available: bool,
    pub reserved: u8,
    pub response_code: u8,
    pub qd_count: u16,
    pub an_count: u16,
    pub nscount: u16,
    pub arcount: u16,
}

#[repr(u8)]
pub enum QueryResponse {
    Query,
    Response,
}

pub struct Question {
    pub name: String,
    pub q_type: Type,
    pub class: Class,
}

#[repr(u16)]
pub enum Type {
    A,
    NS,
    MD,
    MF,
    CNAME,
    SOA,
    MB,
    MG,
    MR,
    NULL,
    WKS,
    PTR,
    HINFO,
    MINFO,
    MX,
    TXT,
}

#[repr(u16)]
pub enum Class {
    IN,
    CS,
    CH,
    HS,
}

impl Header {
    pub fn to_bytes(self) -> BytesMut {
        fn write_u16(s: &mut [u8], val: u16) {
            s[0] = (val >> 8) as u8;
            s[1] = val as u8;
        }

        let mut s = BytesMut::with_capacity(12);
        write_u16(&mut s, self.packet_id);
        s[2] = (self.qr_indicator as u8) << 7
            | self.opcode << 3
            | (self.authoritative_answer as u8) << 2
            | (self.truncation as u8) << 1
            | self.recursion_desired as u8;
        s[3] = (self.recursion_available as u8) << 7 | self.reserved << 4 | self.response_code;
        write_u16(&mut s[4..], self.qd_count);
        write_u16(&mut s[6..], self.an_count);
        write_u16(&mut s[8..], self.nscount);
        write_u16(&mut s[10..], self.arcount);

        s
    }
}

impl Question {
    pub fn serialise(self) -> BytesMut {
        let mut s = BytesMut::with_capacity(self.name.len() + 4);

        for name in self.name.split('.') {
            s.put_u8(name.len() as u8);
            s.put_slice(name.as_bytes());
        }
        s.put_u8(0);
        s.put_u16(self.q_type as u16);
        s.put_u16(self.class as u16);

        s
    }
}
