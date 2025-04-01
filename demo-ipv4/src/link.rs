// Link layer (ethernet)

/// Create the ARP packet to find gateway MAC address.
/// 
/// mac: Our MAC address
/// curr_ip: The current IP address
/// lookup_ip: The IP address we search for it's MAC address
/// 
pub fn create_arp(mac: [u8; 6], curr_ip: [u8; 4], lookup_ip: [u8; 4]) -> Vec<u8> {
    // Set up the ethernet frame
    let mut frame = Vec::new();

    // destination is broadcast
    frame.extend_from_slice(&[0xffu8, 0xff, 0xff, 0xff, 0xff, 0xff]);

    // source
    frame.extend_from_slice(&mac);

    // ethertype 0x0806
    frame.push(0x08);
    frame.push(0x06);

    // hardware address space: Ethernet
    frame.push(0x00);
    frame.push(0x01);

    // protocol address space: IPv4
    frame.push(0x80);
    frame.push(0x00);

    // mac length
    frame.push(0x06);

    // IP length
    frame.push(0x04);

    // opcode: request
    frame.push(0x00);
    frame.push(0x01);

    // source mac
    frame.extend_from_slice(&mac);

    // source address
    frame.extend_from_slice(&curr_ip);

    // dest MAC
    frame.extend_from_slice(&[0x00; 6]);

    // dest IP
    frame.extend_from_slice(&lookup_ip);

    // return frame
    frame
}

/// Gets a ethernet frame, checks if is our ARP response,
/// and if yes, return the MAC address we want to resolve.
/// 
/// frame: Raw ethernet frame
/// mac: Our mac address
/// curr_ip: Our IP address
/// lookup_ip: The IP address we search MAC from
/// 
pub fn unwrap_arp(frame: &[u8], mac: [u8; 6], curr_ip: [u8; 4], lookup_ip: [u8; 4]) -> Option<[u8; 6]> {
    // Check if we are destination
    if frame[0..6] != mac {
        // We are not dest
        return None;
    }

    // Check if is ARP request
    if frame[12..14] != [0x08, 0x06] {
        return None;
    }

    // Check Ethernet/IPv4
    if frame[14..18] != [0x00, 0x01, 0x80, 0x00] {
        return None;
    }

    // Check MAC/IP length and opcode
    if frame[18..22] != [0x06, 0x04, 0x00, 0x02] {
        return None;
    }

    // source MAC (we search)
    let outp: [u8; 6] = frame[22..28].try_into().unwrap();

    // source IP
    if lookup_ip != frame[28..32] {
        return None;
    }

    // dest MAC (our MAC)
    if mac != frame[32..38] {
        return None;
    }

    // dest IP (our IP)
    if curr_ip != frame[38..42] {
        return None;
    }

    // All okay, so return requested MAC
    Some(outp)
}