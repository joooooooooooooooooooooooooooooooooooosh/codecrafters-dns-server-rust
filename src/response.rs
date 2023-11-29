use std::{
    env::args,
    net::{Ipv4Addr, UdpSocket},
};

use anyhow::{Context, Result};
use bytes::Bytes;

use crate::*;

pub fn find_resolver() -> String {
    args()
        .skip_while(|arg| arg != "--resolver")
        .skip(1)
        .next()
        .expect("Should have a resolver ip")
}

pub fn forward(resolver: &String, header: &Header, question: &Question) -> Result<(u16, Bytes)> {
    let udp_socket =
        UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).expect("Failed to bind to address");

    let mut response = header.to_bytes();
    response.extend_from_slice(&question.to_bytes());
    udp_socket.send_to(&response, resolver)?;

    let mut buf = [0; 512];
    match udp_socket.recv_from(&mut buf) {
        Ok((size, _)) => {
            println!("got {size}");
            let mut received_data = Bytes::copy_from_slice(&buf[0..size]);
            let recvd_header =
                Header::from_bytes(&mut received_data).context("error parsing received header")?;
            println!("qwoifjoifjo");
            let _ = Question::from_bytes(&mut received_data, &Vec::new())
                .context("error parsing received question")?;

            Ok((recvd_header.an_count, received_data))
        }
        Err(_) => todo!(),
    }
}
