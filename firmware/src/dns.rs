/// Captive portal DNS server
///
/// Resolves ALL DNS queries to the device's own IP address.
/// This triggers captive portal detection on phones connecting to the AP.
extern crate alloc;

use alloc::vec::Vec;

use std::net::UdpSocket;
use std::thread;

/// Start the DNS server in a background thread
pub fn start(ip: [u8; 4]) {
    thread::Builder::new()
        .name("dns".into())
        .stack_size(8192)
        .spawn(move || dns_loop(ip))
        .expect("DNS thread spawn failed");
}

fn dns_loop(ip: [u8; 4]) {
    let socket = match UdpSocket::bind("0.0.0.0:53") {
        Ok(s) => s,
        Err(e) => {
            log::error!("DNS bind failed: {}", e);
            return;
        }
    };

    log::info!("DNS captive portal server started");

    let mut buf = [0u8; 512];

    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, src)) => {
                if len >= 12 {
                    let resp = build_response(&buf[..len], &ip);
                    let _ = socket.send_to(&resp, src);
                }
            }
            Err(e) => {
                log::warn!("DNS recv error: {}", e);
            }
        }
    }
}

/// Find where the question section ends in a DNS query.
/// Returns the byte offset after QNAME + QTYPE + QCLASS.
fn find_question_end(query: &[u8]) -> usize {
    let mut pos = 12; // Skip 12-byte header
    // Skip QNAME (sequence of length-prefixed labels, terminated by 0)
    while pos < query.len() {
        let label_len = query[pos] as usize;
        if label_len == 0 {
            pos += 1; // Skip null terminator
            break;
        }
        pos += 1 + label_len; // Skip length byte + label data
    }
    pos += 4; // Skip QTYPE (2 bytes) + QCLASS (2 bytes)
    pos.min(query.len())
}

/// Build a DNS A-record response pointing all queries to our IP
fn build_response(query: &[u8], ip: &[u8; 4]) -> Vec<u8> {
    let question_end = find_question_end(query);
    let mut resp = Vec::with_capacity(question_end + 28);

    // Header
    resp.extend_from_slice(&query[..2]); // Transaction ID
    resp.extend_from_slice(&[0x81, 0x80]); // Flags: response, authoritative, no error
    resp.extend_from_slice(&[0x00, 0x01]); // Questions: 1
    resp.extend_from_slice(&[0x00, 0x01]); // Answers: 1
    resp.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // Authority + Additional: 0

    // Question section (only the first question, properly parsed)
    resp.extend_from_slice(&query[12..question_end]);

    // Answer: pointer to name in question, A record, IN class, TTL 60, our IP
    resp.extend_from_slice(&[0xC0, 0x0C]); // Name pointer to offset 12
    resp.extend_from_slice(&[0x00, 0x01]); // Type A
    resp.extend_from_slice(&[0x00, 0x01]); // Class IN
    resp.extend_from_slice(&[0x00, 0x00, 0x00, 0x3C]); // TTL 60s
    resp.extend_from_slice(&[0x00, 0x04]); // Data length 4
    resp.extend_from_slice(ip); // IP address

    resp
}
