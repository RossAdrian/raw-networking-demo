// Link layer (ethernet)
use std::net::{Ipv6Addr, SocketAddrV6};

use crate::ip::icmpv6_check_neighbor;

/// Create Ethernet packet over IPv6
/// 
pub fn create_ethernet_packet(src_mac: &[u8; 6], dest_mac: &[u8; 6], src: SocketAddrV6, dest: SocketAddrV6) -> Vec<u8> {
    // Create ethernet frame
    let mut frame = Vec::new();
    
    // push dest/src MAC
    frame.extend_from_slice(dest_mac);
    frame.extend_from_slice(src_mac);

    // push Ethertype 0x86DD (IPv6)
    frame.push(0x86);
    frame.push(0xDD);

    // Create payload of IPv6 packet
    crate::ip::create_ip_packet(&mut frame, src, dest);

    frame
}

/// Unpack ethernet packet
///
pub fn unpack_ethernet_packet(frame: &[u8], our_mac: &[u8; 6], gateway_mac: &[u8; 6], src: SocketAddrV6, dest: SocketAddrV6) -> Option<String> {
    // check we are destination
    if &frame[0..6] != our_mac {
        return None;
    }

    // check source
    if &frame[6..12] != gateway_mac {
        return None;
    }

    // Check ethertype
    if frame[12..14] != [0x86, 0xDD] {
        return None;
    }

    // Okay, delegate to IP layer
    crate::ip::unpack(&frame[14..], src, dest)
}

/// Listen for ICMPv6 packet router advertisement.
/// 
pub fn unpack_icmp(frame: &[u8]) -> Option<[u8; 6]> {
    // Check ethertype
    if frame[12..14] != [0x86, 0xDD] {
        return None;
    }

    // Save possible source (Router?)
    let src: [u8; 6] = frame[6..12].try_into().expect("Could not get source MAC");

    // Check if is router, and if yes, return MAC
    if icmpv6_check_neighbor(&frame[14..]) {
        Some(src)
    }else{
        None
    }
}

/// Create ICMPv6 router solicitation packet.
/// 
pub fn create_eth_router_solicitation(our_mac: &[u8; 6], ip: &Ipv6Addr) -> Vec<u8> {
    // Create ethernet frame
    let mut frame = Vec::new();
    
    // push dest/src MAC
    frame.extend_from_slice(&[0x33, 0x33, 0x00, 0x00, 0x00, 0x02]);
    frame.extend_from_slice(our_mac);

    // push Ethertype 0x86DD (IPv6)
    frame.push(0x86);
    frame.push(0xDD);

    // Create ip layer ICMPv6 payload
    crate::ip::icmpv6_create_router_solicitation(&mut frame, ip);

    frame
}