use anyhow::{Context, Result};
use bytes::Bytes;
use dns_starter_rust::*;
use std::net::UdpSocket;

fn main() -> Result<()> {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                let mut received_data = Bytes::copy_from_slice(&buf[0..size]);
                // println!("Received {} bytes from {}", size, source);
                // println!("{received_data:?}");

                let header = Header::from_bytes(&mut received_data)
                    .context("error parsing recieved header")?;

                let response = Header {
                    packet_id: header.packet_id,
                    qr_indicator: QueryResponse::Response,
                    opcode: header.opcode,
                    authoritative_answer: false,
                    truncation: false,
                    recursion_desired: header.recursion_desired,
                    recursion_available: false,
                    reserved: 0,
                    response_code: if header.opcode == 0 {
                        ResponseCode::NoError
                    } else {
                        ResponseCode::NotImplemented
                    },
                    qd_count: 1,
                    an_count: 1,
                    ns_count: 0,
                    ar_count: 0,
                };

                let domain = String::from("codecrafters.io");
                let question = Question {
                    name: domain.clone(),
                    q_type: Type::A,
                    class: Class::IN,
                };

                let answer = Answer {
                    name: domain,
                    a_type: Type::A,
                    class: Class::IN,
                    ttl: 60,
                    rdlength: 4,
                    data: 23983289,
                };

                let mut response = response.to_bytes();
                response.extend_from_slice(&question.to_bytes());
                response.extend_from_slice(&answer.to_bytes());
                udp_socket
                    .send_to(&response, source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }

    Ok(())
}
