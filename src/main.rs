use anyhow::{Context, Result};
use bytes::Bytes;
use std::net::UdpSocket;

use dns_starter_rust::*;

fn main() -> Result<()> {
    let resolver = response::find_resolver();
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                let mut received_data = Bytes::copy_from_slice(&buf[0..size]);

                let recvd_header = Header::from_bytes(&mut received_data)
                    .context("error parsing received header")?;
                let forward_header = Header {
                    qd_count: 1,
                    ..recvd_header
                };

                let mut header = Header {
                    packet_id: recvd_header.packet_id,
                    qr_indicator: QueryResponse::Response,
                    opcode: recvd_header.opcode,
                    authoritative_answer: false,
                    truncation: false,
                    recursion_desired: recvd_header.recursion_desired,
                    recursion_available: false,
                    reserved: 0,
                    response_code: if recvd_header.opcode == 0 {
                        ResponseCode::NoError
                    } else {
                        ResponseCode::NotImplemented
                    },
                    qd_count: recvd_header.qd_count,
                    an_count: 0,
                    ns_count: 0,
                    ar_count: 0,
                };

                let mut questions = Vec::with_capacity(recvd_header.qd_count as usize);
                let mut answers = Vec::with_capacity(recvd_header.qd_count as usize);
                for q_num in 0..recvd_header.qd_count {
                    let question = Question::from_bytes(&mut received_data, &questions)
                        .context(format!("error parsing received question #{q_num}"))?;

                    let (an_count, answer) =
                        response::forward(&resolver, &forward_header, &question)?;
                    header.an_count += an_count;
                    answers.push(answer);

                    questions.push(question);
                }

                let mut response = header.to_bytes();
                response.extend_from_slice(&buf[HEADER_LENGTH..size]);
                for answer in answers {
                    response.extend_from_slice(&answer)
                }

                // for question in questions {
                //     let answer = Answer {
                //         name: question.name.clone(),
                //         a_type: Type::A,
                //         class: Class::IN,
                //         ttl: 60,
                //         rdlength: 4,
                //         data: 23983289,
                //     };

                //     response.extend_from_slice(&answer.to_bytes());
                // }

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
