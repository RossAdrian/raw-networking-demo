// IP layer
use std::net::{Ipv6Addr, SocketAddrV6};

use crate::udp::compute_checksum;

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
    data[plen_idx+1] = len[1];
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

/// Create a ICMPv6 router solicitation packet.
/// 
/// This packet is required for discover the gateway.
/// 
pub fn icmpv6_create_router_solicitation(data: &mut Vec<u8>, our_ip: &Ipv6Addr) {
    // push version 6, traffic class 0, flow label 0
    data.extend_from_slice(&[6u8 << 4, 0, 0, 0]);

    let ipv6_len_idx = data.len();

    // push for now payload length 0, next header 58=ICMPv6, Hop limit=255
    data.extend_from_slice(&[0, 0, 58, 0xff]);

    // source address octets
    data.extend_from_slice(&our_ip.octets());

    // dest address octets (all router link local multicast)
    data.extend_from_slice(&"ff02::2".parse::<Ipv6Addr>().unwrap().octets());

    // Now the ICMPv6 payload
    // keep track of current index for later checksum calculation
    let idx_icmpv6_begin = data.len();

    // ICMPv6 type Router Solicitation=133
    data.push(133);

    // code=0, checksum 0 for now + 4 bytes reserved=0
    data.extend_from_slice(&[0; 7]);

    // Now calculate checksum (RFC 2460, Section 8.1)
    let mut check = Vec::new();

    // IPv6 pseodo header (RFC 2460, Section 8.1)
    // Source IPv6 address
    check.extend_from_slice(&our_ip.octets());

    // Destination IPv6 address
    check.extend_from_slice(&"ff02::2".parse::<Ipv6Addr>().unwrap().octets());

    // Set IPv6 length
    let len = (data[idx_icmpv6_begin..].len() as u16).to_be_bytes();

    // u32 ICMPv6 message length
    check.extend_from_slice(&len);

    // 3x zero + ICMPv6 type (58)
    check.extend_from_slice(&[0,0,0,58]);

    // push payload
    check.extend_from_slice(&data[idx_icmpv6_begin..]);

    // padd
    if check.len() % 2 == 1 {
        check.push(0x00);
    }

    // calculate checksum
    let check = compute_checksum(&check).to_be_bytes();

    // Set IPv6 length
    data[ipv6_len_idx] = len[0];
    data[ipv6_len_idx+1] = len[1];

    // Set ICMPv6 checksum
    data[idx_icmpv6_begin+2] = check[0];
    data[idx_icmpv6_begin+3] = check[1];
}