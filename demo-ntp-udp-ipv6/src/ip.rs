// IP layer
use std::net::SocketAddrV6;

/// Creates the IPv6 packet
pub fn create_ip_packet(data: &mut Vec<u8>, src: SocketAddrV6, dest: SocketAddrV6) {
    // push version 6, traffic class 0, flow label 0
    data.extend_from_slice(&[6u8 << 4, 0, 0, 0]);

    // store payload length index for later use
    let plen_idx = data.len();

    // push for now payload length 0, next header 17=UDP, Hop limit 60
    data.extend_from_slice(&[0, 0, 17, 60]);

    // source address octets
    data.extend_from_slice(&src.ip().octets());

    // dest address octets
    data.extend_from_slice(&dest.ip().octets());

    // put on the UDP frame
    crate::udp::create_packet(data, dest, src);

    // compute length
    let len = u16::try_from(data[{plen_idx+36}..].len()).expect("IPv6 payload too long.").to_be_bytes();

    data[plen_idx] = len[0];
    data[plen_idx] = len[1];
    // Ready
}

/// Unpack IPv6 packet.
pub fn unpack(frame: &[u8], src: SocketAddrV6, dest: SocketAddrV6) -> Option<String> {
    // check we are addressed
    if &frame[8..24] != &src.ip().octets() {
        return None;
    }

    if &frame[24..40] != &dest.ip().octets() {
        return None;
    }

    // Check IP protocol number
    if frame[6] != 17 {
        return None;
    }

    // Now check on the next layer
    crate::udp::unpack(&frame[40..], src, dest)
}

/// Check if is ICMPv6 neighbor advertisement from the router
/// 
/// Returns true, if this is a router, and the link layer should use the MAC for futher communication.
/// 
pub fn icmpv6_check_neighbor(frame: &[u8]) -> bool {
    // Check if ICMPv6
    if frame[6] != 58 {
        return false;
    }

    // Check if router advertisement
    if frame[40] == 134 {
        return true;

    // if is not neighbor advertisement, return false
    }else if frame[40] != 136 {
        return false;
    }

    // if is neighbor advertisement, check if router bit set
    if frame[44] & 0x80 != 0 {
        true
    }else{
        false
    }
}