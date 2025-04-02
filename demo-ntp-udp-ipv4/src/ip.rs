// IP layer
use std::net::{IpAddr, SocketAddr};

use crate::udp::create_packet;

/// Creates the IPv4 packet
pub fn create_ip_packet(src: SocketAddr, dest: SocketAddr) -> Vec<u8> {
    let mut data = Vec::new();

    // calculate header

    // first byte (0x45)
    data.push( (4u8 << 4) | 5 );
    data.push(0);

    // length on index 2 and 3 will be set later
    data.push(0);
    data.push(0);

    // ID, flags, offset (all ignored)
    // Just use ID 0xcc80
    data.push(0xcc);
    data.push(0x80);
    data.push(0);
    data.push(0);

    // TTL 60
    data.push(60);

    // protocol UDP
    data.push(17);

    // checksum will be done later
    data.push(0);// index 10
    data.push(0);// index 11

    // source address
    let addr = if let IpAddr::V4(addr) = src.ip() {
        addr
    }else{
        panic!("Wrong address")
    };

    data.extend_from_slice( &addr.octets() );

    // target address
    let addr = if let IpAddr::V4(addr) = dest.ip() {
        addr
    }else{
        panic!("Wrong address")
    };

    data.extend_from_slice( &addr.octets() );

    // the payload
    data.extend(create_packet(dest.port(), src.port()));

    // calculate total length
    let len = u16::try_from(data.len()).expect("Payload too long").to_be_bytes();
    data[2] = len[0];
    data[3] = len[1];

    // Now calculate checksum over IP header
    let checksum = compute_checksum(&data[0..20]).to_be_bytes();
    data[10] = checksum[0];
    data[11] = checksum[1];

    // ready, return data
    data

}

pub fn compute_checksum(header: &[u8]) -> u16 {
    // Ensure the header length is even (for 16-bit processing)
    if header.len() % 2 != 0 {
        panic!("Header length must be even");
    }

    // Sum all 16-bit words
    let mut sum: u32 = 0;
    for chunk in header.chunks_exact(2) {
        let word = u16::from_be_bytes([chunk[0], chunk[1]]);
        sum += u32::from(word);
    }

    // Add the carry bits
    while sum > 0xFFFF {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    // One's complement of the sum
    !(sum as u16)
}

pub fn unpack(frame: &[u8], src: SocketAddr, dest: SocketAddr) -> Option<String> {
    // calculate checksum
    let header_len = ((frame[0] & 0x0F) * 4) as usize;
    let mut header = Vec::new();
    frame[0..{header_len}].clone_into(&mut header);

    header[10] = 0;
    header[11] = 0;

    let checksum = compute_checksum(header.as_slice()).to_be_bytes();
    if checksum != frame[10..=11] {
        panic!("Received corrupted IP packet, {:?} != {:?}", checksum, &frame[10..=11]);
    }

    // contine checking

    // check if UDP
    if frame[9] != 17 {
        return None;
    }

    // check src and dest address
    let ip_src: [u8; 4] = frame[12..16].try_into().unwrap();
    let ip_dest: [u8; 4] = frame[16..20].try_into().unwrap();

    // compare
    let srca = if let IpAddr::V4(addr) = src.ip() {
        addr
    }else{
        panic!("Unreachable");
    };

    let desta = if let IpAddr::V4(addr) = dest.ip() {
        addr
    }else{
        panic!("Unreachable");
    };

    if srca.octets() == ip_src && desta.octets() == ip_dest {
        crate::udp::unpack(&frame[{header_len}..], src.port(), dest.port())
    }else{
        None
    }
}