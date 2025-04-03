use std::{net::{IpAddr, Ipv4Addr, SocketAddr}, process::exit};

use link::{create_arp, create_ethernet_packet, unwrap_arp, unwrap_ethernet_packet};
use pnet::datalink::{self, Channel};
use std::time::Instant;

mod ntp; // Application layer
mod udp; // Transport layer
mod ip;  // IP layer
mod link;// Link layer

/// The main function.
/// 
/// Command line arguments:
/// 
/// 1. Interface name (The ethernet interface to use, normaly something like eth0 or enp1s0)
/// 2. Gateway IP (Commonly something like 192.168.x.1)
/// 3. Bind port (some open port to bind our host on)
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

    // Get and parse commandline arguments

    // gateway IP
    let gateway_ip = {
        let ip: Ipv4Addr = std::env::args().nth(2).expect("Expected gateway IP").parse().expect("Expected IPv4 address");
        ip
    };

    let port = std::env::args().nth(3).expect("Expected binding port");
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
        let i: Vec<_> = interface.ips.iter().filter(|i| i.is_ipv4()).collect();
        let i = i.get(0).expect("Interface has no IP address assigned").ip();

        if let IpAddr::V4(addr) = i {
            addr
        }else{
            println!("Wrong IP address found, expected IPv4 address.");
            exit(0);
        }
    };

    // Create the channel
    let channel = datalink::channel(&interface, Default::default());
    let (mut tx,mut rx) = match channel {
        Ok(Channel::Ethernet(tx,rx)) => (tx,rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("Error when creating raw socket: {}",e),
    };

    // first, setup ARP request to get gateway MAC
    let gateway_mac = {
        let mut res;

        // Create ARP packet to send
        let packet = create_arp(this_mac.octets(), this_ip.octets(), gateway_ip.octets());

        // send ARP packet
        tx.send_to(&packet, None);

        // current time
        let t_prev = Instant::now();

        // loop to get requests
        loop {
            match rx.next() {
                Ok(frame) => {
                    // try to parse ethernet frame.
                    res = unwrap_arp(frame, this_mac.octets(), this_ip.octets(), gateway_ip.octets());
                },
                Err(e) => {
                    panic!("Error occured: {}.", e);
                }
            }

            // break loop if we found mac
            if let Some(_) = res {
                break;
            }

            // if not, check if timeout, else continue
            if (Instant::now() - t_prev).as_secs_f32() > 3.0 {
                // timeout
                println!("ARP request timeout!");
                exit(-1);
            }
        }

        // safety: It would be never callen if not satisfied above as Some(_)
        res.unwrap()
    };

    // Now send ethernet packet
    let timestamp = {
        let mut res;

        let dest: SocketAddr = format!("216.239.35.12:{}", 123).parse().expect("Could not resolve time.google.com");
        let src = SocketAddr::new(IpAddr::V4(this_ip), port);

        let packet = create_ethernet_packet(&this_mac.octets(), &gateway_mac, src, dest);

        // Send NTP packet
        tx.send_to(&packet, None);

        // current time
        let t_prev = Instant::now();

        // loop to get requests
        loop {
            match rx.next() {
                Ok(frame) => {
                    // try to parse bottom-up the NTP packet to get timestamp
                    res = unwrap_ethernet_packet(frame, &this_mac.octets(), &gateway_mac, dest, src);
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
    println!("Usage:    cargo run --release --bin demo-ipv4 <interface> <gateway> <UDP port>\n");
    println!("  --help, -h    Display this help message");
    println!("  <interface>   The ethernet interface to send through.");
    println!("  <gateway>     The gateway IP address. The IPv4 address of your internet router.");
    println!("                Normally something like 192.168.x.1 or similar.");
    println!("  <UDP port>    Port to bind the UDP socket on (or better said steel the port).\n");
    println!("Bug reports issue at https://github.com/RossAdrian/raw-networking-demo/.");


    exit(code)
}