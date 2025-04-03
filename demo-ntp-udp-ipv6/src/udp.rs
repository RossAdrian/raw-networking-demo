use std::net::SocketAddrV6;

// UDP (transport layer)
use crate::ntp::*;

/// Create the UDP packet
pub fn create_packet(data: &mut Vec<u8>, dest: SocketAddrV6, src: SocketAddrV6) {
    // Used for checksum
    let curr_idx = data.len();
    // source port
    data.extend_from_slice(&src.port().to_be_bytes());

    // destination port
    data.extend_from_slice(&dest.port().to_be_bytes());

    // length and checksum currently ignored
    data.extend_from_slice(&[0x00u8; 4]);
    
    // setup ntp payload
    add_request_payload(data);

    // set length
    let length = u16::try_from(data[curr_idx..].len()).expect("Payload to big");
    let bytes = length.to_be_bytes();
    data[curr_idx + 4] = bytes[0];
    data[curr_idx + 5] = bytes[1];

    // Calculate checksum
    let mut check = Vec::new();

    // create IP pseodo header
    check.extend_from_slice(&src.ip().octets());
    check.extend_from_slice(&dest.ip().octets());
    check.extend_from_slice(&(data[curr_idx..].len() as u32).to_be_bytes());
    check.extend_from_slice(&[0; 3]);
    check.push(17);

    // push UDP header and payload
    check.extend_from_slice(&data[curr_idx..]);

    if check.len() % 2 == 1 {
        // align 2
        check.push(0x00);
    }

    // calculate checksum
    let checksum = compute_checksum(check.as_slice()).to_be_bytes();

    // set checksum
    data[curr_idx + 6] = checksum[0];
    data[curr_idx + 7] = checksum[1];
}

/// Unwraps the UDP packet and return the NTP timestamp
pub fn unpack(data: &[u8], src: SocketAddrV6, dest: SocketAddrV6) -> Option<String> {
    // First: Verify checksum
    let mut check = Vec::new();

    check.extend_from_slice(&src.ip().octets());// 16
    check.extend_from_slice(&dest.ip().octets());// 32
    check.extend_from_slice(&(data.len() as u32).to_be_bytes());// 36
    check.extend_from_slice(&[0; 3]);// 39
    check.push(17);// 40
    // End IPv6 pseodo header

    // push UDP header and payload
    check.extend_from_slice(data);

    // remove checksum in check array
    // must be at 46 + 47
    check[46] = 0;
    check[47] = 0;

    // padd if required
    if check.len() % 2 == 1 {
        // align 2
        check.push(0x00);
    }

    // calculate checksum
    let checksum = compute_checksum(check.as_slice()).to_be_bytes();

    // Check checksum
    if &checksum != &data[6..=7] {
        println!("Received corrupted IPv6/UDP packet: Dropping it.");
        return None;
    }

    // check port
    let src = src.port().to_be_bytes();
    let dest = dest.port().to_be_bytes();

    if src == data[0..2] && dest == data[2..4] {
        // ignore checksum, length, ..., just delegate payload to NTP
        Some(get_timestamp(&data[8..]))
    }else{
        None
    }    
}

/// Checksum compute function.
pub fn compute_checksum(data: &[u8]) -> u16 {
    // Ensure the data length is even (for 16-bit processing)
    if data.len() % 2 != 0 {
        panic!("Data length must be even");
    }

    // Sum all 16-bit words
    let mut sum: u32 = 0;
    for chunk in data.chunks_exact(2) {
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