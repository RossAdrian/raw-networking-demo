use std::{net::{IpAddr, SocketAddrV6}, process::exit, time::Instant};

use link::{create_eth_router_solicitation, create_ethernet_packet};
use pnet::datalink::{self, Channel};

mod ntp; // Application layer
mod udp; // Transport layer
mod ip;  // IP layer
mod link;// Link layer

/// The main function.
/// 
/// Commandline arguments:
/// 
/// 1. Interface name (The ethernet interface to use, normaly something like eth0 or enp1s0)
/// 2. Binding port (some open port to bind our host on)
/// 
/// # Security
/// 
/// Do not consider this as a security note for rust safty.
/// 
/// Consider this rather as a operating system security note.
/// 
/// Do make sure the second parameter, the binding port number,
/// is free and not allocated while usage of this program. Since
/// we kind of *steal* the port, that would be allocated and
/// managed by the operating system, it could conflict with other
/// services that currently uses this port, and cause undefined
/// behavior.
/// 
fn main() {
    // Get interface name, or else display help and return
    let interface_name = if let Some(s) = std::env::args().nth(1) {
        if s == "--help" || s == "-h" {
            help_message(0);
        }

        String::from(s)
    }else{
        help_message(1);
    };

    // Port to bind UDP on
    let port = std::env::args().nth(2).expect("Expected binding port");
    let port: u16 = port.parse().expect("Cannot parse port");

    // Get the interface we search
    let interface = {
        let interfaces = pnet::datalink::interfaces();
        let mut interface = None;

        for i in interfaces {
            if i.name == interface_name {
                interface = Some(i);
                break;
            }
        }

        if let Some(i) = interface {
            i
        }else{
            println!("{}: No such interface.", interface_name);
            exit(-1);
        }
    };

    let this_mac = interface.mac.expect("Interface has no mac address");
    let this_ip = {
        let i: Vec<_> = interface.ips.iter().filter(|i| i.is_ipv6() && if let IpAddr::V6(addr) = i.ip() {
            addr.octets()[0] & 0x20 != 0// Search for 2000::/3 address
        }else{
            false
        }).collect();
        let ix: Vec<_> = i.iter().map(|ip| if let IpAddr::V6(addr) = ip.ip() {
            addr
        }else{
            println!("Wrong IP address found, expected IPv6 address.");
            exit(0);
        }).collect();

        ix.get(0).expect("Interface has no IP address assigned").clone()
    };

    // Create the channel
    let channel = datalink::channel(&interface, Default::default());
    let (mut tx,mut rx) = match channel {
        Ok(Channel::Ethernet(tx,rx)) => (tx,rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("Error when creating raw socket: {}",e),
    };

    // first listen for router advertisement
    let gateway_mac = {
        let mut res;

        let packet = create_eth_router_solicitation(&this_mac.octets(), &this_ip);

        // send icmp packet
        tx.send_to(&packet, None);

        // current time
        let t_prev = Instant::now();

        loop {
            match rx.next() {
                Ok(frame) => {
                    res = crate::link::unpack_icmp(frame);
                },
                Err(e) => {
                    panic!("Error occured: {}.", e);
                }
            }

            // break look if we found MAC
            if let Some(_) = res {
                break;
            }

            // if not, check if timeout, else continue
            if (Instant::now() - t_prev).as_secs_f32() > 7.0 {
                // timeout
                println!("Router advertisement listening timeout!\nDoes your router implement RFC4861?");
                exit(-1);
            }
        }

        // safty: It would be never callen if not satisfied above as Some(_)
        res.unwrap()
    };

    // Now send ethernet packet
    let timestamp = {
        let mut res;

        let dest: SocketAddrV6 = format!("[2001:4860:4806:4::]:{}", 123).parse().expect("Could not resolve time.google.com");
        let src = SocketAddrV6::new(this_ip, port, 0, 0);

        let packet = create_ethernet_packet(&this_mac.octets(), &gateway_mac, src, dest);

        // Send NTP packet
        tx.send_to(&packet, None);

        let t_prev = Instant::now();

        // loop to get requests
        loop {
            match rx.next() {
                Ok(frame) => {
                    // try to parse bottom-up the NTP packet to get timestamp
                    res = crate::link::unpack_ethernet_packet(frame, &this_mac.octets(), &gateway_mac, dest, src);
                },
                Err(e) => {
                    panic!("Error occured: {}", e)
                }
            }

            // break loop if we found timestamp
            if let Some(_) = res {
                break;
            }

            // Check if timeout, else continue
            if (Instant::now() - t_prev).as_secs_f32() > 7.0 {
                // timeout
                println!("NTP request timeout!");
                exit(-1);
            }
        }

        // safety: It would be never callen if not satisfied above as Some(_)
        res.unwrap()
    };

    println!("Current time (UTC): {}", timestamp);
}

/// Help message.
///
fn help_message(code: i32) -> ! {
    println!("Usage:    cargo run --release --bin demo-ipv4 <interface> <UDP port>\n");
    println!("  --help, -h    Display this help message");
    println!("  <interface>   The ethernet interface to send through.");
    println!("  <UDP port>    Port to bind the UDP socket on (or better said steel the port).\n");
    println!("Bug reports issue at https://github.com/RossAdrian/raw-networking-demo/.");


    exit(code)
}