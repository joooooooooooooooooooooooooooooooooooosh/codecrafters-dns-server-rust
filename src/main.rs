use dns_starter_rust::*;
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
                    question_count: 0,
                    an_count: 0,
                    nscount: 0,
                    arcount: 0,
                };
                udp_socket
                    .send_to(&response.to_bytes(), source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
