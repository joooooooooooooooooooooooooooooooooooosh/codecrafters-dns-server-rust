use dns_starter_rust::*;
use nom::AsBytes;
use std::net::UdpSocket;

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                let _received_data = String::from_utf8_lossy(&buf[0..size]);
                println!("Received {} bytes from {}", size, source);
                let response = Header {
                    packet_id: 1234,
                    qr_indicator: QueryResponse::Response,
                    opcode: 0,
                    authoritative_answer: false,
                    truncation: false,
                    recursion_desired: false,
                    recursion_available: false,
                    reserved: 0,
                    response_code: 0,
                    qd_count: 1,
                    an_count: 1,
                    nscount: 0,
                    arcount: 0,
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
                response.extend_from_slice(question.serialise().as_bytes());
                response.extend_from_slice(answer.serialise().as_bytes());
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
}
