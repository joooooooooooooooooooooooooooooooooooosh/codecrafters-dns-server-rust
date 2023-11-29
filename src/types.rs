use std::mem::size_of;

use bytes::{Buf, BufMut, Bytes, BytesMut};

pub static HEADER_LENGTH: usize = 12;

pub struct Header {
    pub packet_id: u16,
    pub qr_indicator: QueryResponse,
    pub opcode: u8,
    pub authoritative_answer: bool,
    pub truncation: bool,
    pub recursion_desired: bool,
    pub recursion_available: bool,
    pub reserved: u8,
    pub response_code: ResponseCode,
    pub qd_count: u16,
    pub an_count: u16,
    pub ns_count: u16,
    pub ar_count: u16,
}

#[repr(u8)]
pub enum ResponseCode {
    NoError = 0,
    FormatError,
    ServerFailure,
    NameError,
    NotImplemented,
    Refused,
}

impl TryFrom<u8> for ResponseCode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NoError),
            1 => Ok(Self::FormatError),
            2 => Ok(Self::ServerFailure),
            3 => Ok(Self::NameError),
            4 => Ok(Self::NotImplemented),
            5 => Ok(Self::Refused),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum QueryResponse {
    Query = 0,
    Response,
}

impl TryFrom<u8> for QueryResponse {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Query),
            1 => Ok(Self::Response),
            _ => Err(()),
        }
    }
}

pub struct Question {
    pub name: String,
    pub q_type: Type,
    pub class: Class,
    offset: usize,
    len: usize,
}

#[derive(Clone, Copy)]
#[repr(u16)]
pub enum Type {
    A = 1,
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

impl TryFrom<u16> for Type {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Type::A),
            2 => Ok(Type::NS),
            3 => Ok(Type::MD),
            4 => Ok(Type::MF),
            5 => Ok(Type::CNAME),
            6 => Ok(Type::SOA),
            7 => Ok(Type::MB),
            8 => Ok(Type::MG),
            9 => Ok(Type::MR),
            10 => Ok(Type::NULL),
            11 => Ok(Type::WKS),
            12 => Ok(Type::PTR),
            13 => Ok(Type::HINFO),
            14 => Ok(Type::MINFO),
            15 => Ok(Type::MX),
            16 => Ok(Type::TXT),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy)]
#[repr(u16)]
pub enum Class {
    IN = 1,
    CS,
    CH,
    HS,
}

impl TryFrom<u16> for Class {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Class::IN),
            2 => Ok(Class::CS),
            3 => Ok(Class::CH),
            4 => Ok(Class::HS),
            _ => Err(()),
        }
    }
}

pub struct Answer {
    pub name: String,
    pub a_type: Type,
    pub class: Class,
    pub ttl: u32,
    pub rdlength: u16,
    pub data: u32, // TODO: data type depends on a_type
}

impl Header {
    pub fn to_bytes(&self) -> BytesMut {
        let mut s = BytesMut::with_capacity(12);
        s.put_u16(self.packet_id);
        s.put_u8(
            (self.qr_indicator as u8) << 7
                | self.opcode << 3
                | (self.authoritative_answer as u8) << 2
                | (self.truncation as u8) << 1
                | self.recursion_desired as u8,
        );
        s.put_u8(
            (self.recursion_available as u8) << 7 | self.reserved << 4 | self.response_code as u8,
        );
        s.put_u16(self.qd_count);
        s.put_u16(self.an_count);
        s.put_u16(self.ns_count);
        s.put_u16(self.ar_count);

        s
    }

    pub fn from_bytes(src: &mut Bytes) -> Option<Self> {
        let packet_id = src.get_u16();

        let next = src.get_u8();
        let qr_indicator = (next >> 7 & 0b1).try_into().ok()?;
        let opcode = (next >> 3) & 0b1111;
        let authoritative_answer = ((next >> 2) & 0b1) != 0;
        let truncation = ((next >> 1) & 0b1) != 0;
        let recursion_desired = (next & 0b1) != 0;

        let next = src.get_u8();
        let recursion_available = (next >> 7 & 0b1) != 0;
        let reserved = (next >> 4) & 0b111;
        let response_code = (next & 0b1111).try_into().ok()?;

        let qd_count = src.get_u16();
        let an_count = src.get_u16();
        let ns_count = src.get_u16();
        let ar_count = src.get_u16();

        Some(Self {
            packet_id,
            qr_indicator,
            opcode,
            authoritative_answer,
            truncation,
            recursion_desired,
            recursion_available,
            reserved,
            response_code,
            qd_count,
            an_count,
            ns_count,
            ar_count,
        })
    }
}

impl Question {
    pub fn to_bytes(&self) -> BytesMut {
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

    pub fn from_bytes(src: &mut Bytes, questions: &Vec<Question>) -> Option<Self> {
        let start_len = src.len();

        let mut len = src.get_u8();
        let mut name = String::with_capacity(len as usize);
        while len != 0 {
            if len & 0b1100_0000 != 0 {
                let len = len & (0b0011_1111);
                let pointer = (((len as u16) << 8) | src.get_u8() as u16).into();
                let question = questions.iter().rev().find(|q| q.offset < pointer)?;
                let domain_offset = pointer - question.offset;
                name.push_str(&question.name[domain_offset..]);

                break;
            }

            for _ in 0..len {
                name.push(src.get_u8() as char)
            }

            len = src.get_u8();
            if len != 0 {
                name.push('.')
            }
        }

        let q_type = src.get_u16().try_into().ok()?;
        let class = src.get_u16().try_into().ok()?;

        Some(Self {
            name,
            q_type,
            class,
            offset: questions
                .last()
                .and_then(|q| Some(q.offset + q.len))
                .unwrap_or(HEADER_LENGTH),
            len: start_len - src.len(),
        })
    }
}

impl Answer {
    pub fn to_bytes(&self) -> BytesMut {
        let mut s = BytesMut::with_capacity(size_of::<Answer>());

        for name in self.name.split('.') {
            s.put_u8(name.len() as u8);
            s.put_slice(name.as_bytes());
        }
        s.put_u8(0);

        s.put_u16(self.a_type as u16);
        s.put_u16(self.class as u16);
        s.put_u32(self.ttl);
        s.put_u16(self.rdlength);
        s.put_u32(self.data);

        s
    }
}
