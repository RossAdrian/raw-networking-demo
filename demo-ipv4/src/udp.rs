// UDP (transport layer)
use crate::ntp::*;

/// Create the UDP packet
pub fn create_packet(dest: u16, src: u16) -> Vec<u8> {
    let mut data = Vec::new();

    // source port
    data.extend_from_slice(&src.to_be_bytes());

    // destination port
    data.extend_from_slice(&dest.to_be_bytes());

    // length and checksum currently ignored
    data.extend_from_slice(&[0x00u8; 4]);
    
    // setup ntp payload
    add_request_payload(&mut data);

    // set length
    let length = u16::try_from(data.len() - 8).expect("Payload to big");
    let bytes = length.to_be_bytes();
    data[4] = bytes[0];
    data[5] = bytes[1];

    // And return the frame
    data
}

/// Unwraps the UDP packet and return the NTP timestamp
pub fn unpack(data: Vec<u8>) -> String {
    // ignore checksum, length, ..., just delegate payload to NTP
    get_timestamp(&data[8..])
}