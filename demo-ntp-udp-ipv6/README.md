# NTP over UDP/IPv6 Demo

A demonstration of raw socket programming that implements a complete networking stack to query time.google.com using the NTP protocol.

## Overview

This crate demonstrates how a simple NTP time request works at every layer of the networking stack by implementing the entire process from raw ethernet frames up. It provides insights into what happens under the hood when you make a network time request.

### Networking Stack Implementation

The implementation follows the ISO/OSI model from bottom to top:

1. **Link Layer** (`link.rs`)
   - Raw Ethernet frame crafting
   
2. **Internet Layer** (`ip.rs`) 
   - IPv6 packet handling
   - IP header construction
   - ICMPv6 Router solicitation for router discovery

3. **Transport Layer** (`udp.rs`)
   - UDP datagram handling
   - Port management

4. **Application Layer** (`ntp.rs`)
   - NTP protocol implementation
   - Time request/response handling

## Usage

The program requires root privileges to create raw sockets. Run it using:

```bash
cargo run --release -- <interface> <UDP port>
```

## Arguments

- `<interface>`: The ethernet interface to send through (e.g. eth0 or enp1s0)
- `<UDP port>`: Local UDP port to bind to

## Example

```bash
cargo run --release -- eth0 12345
```

## Security Note

The specified UDP port must be free and not allocated by other services, as the program bypasses the operating system's normal port management. Using an already allocated port may cause conflicts and undefined behavior.

## Prerequesites

- Linux-based operating system
- Rust toolchain
- Root privileges
- Ethernet interface *UP* with *IPv6* connectivity

## Explanation

The program demonstrates how networking protocols are layered by implementing a complete NTP request/response cycle from the ground up:

1. **Initial Setup**
   - Opens a raw ethernet channel to the specified network interface
   - Retrieves interface MAC address and IPv6 address
   - Prepares for raw packet transmission/reception

2. **MAC Address Resolution**
   - Creates and sends an ICMPv6 Router Solicitation to resolve gateway's MAC address
   - Waits up to 7 seconds for ICMPv6 Router Advertisement
   - Extracts gateway MAC from response for further communication

3. **NTP Request**
   - Constructs packet bottom-up through all layers:
     * Ethernet frame (Link layer)
     * IPv6 packet (Internet layer)
     * UDP datagram (Transport layer)
     * NTP request (Application layer)
   - Sets time.google.com (2001:4860:4806:4::) as destination
   - Sends complete packet through ethernet channel

4. **Response Processing**
   - Listens for incoming packets (7 second timeout)
   - For each received frame, processes bottom-up:
     * Validates ethernet frame addressing
     * Checks IPv6 packet headers
     * Verifies UDP port numbers
     * Extracts NTP timestamp from payload
   - Displays received timestamp when valid response found

This implementation shows how each networking layer adds its own headers and addressing information, demonstrating the encapsulation process that normally happens within the operating system's networking stack.

## Relevant Sources

- [RFC 4861, Section 4](https://datatracker.ietf.org/doc/html/rfc4861#section-4): Router Solicitation/Advertisement format
- [RFC 2460](https://datatracker.ietf.org/doc/html/rfc2460): Internet Protocol, Version 6 (IPv6) Specification
- [RFC 768](https://datatracker.ietf.org/doc/html/rfc768): User Datagram Protocol (UDP)
- [RFC 5905](https://datatracker.ietf.org/doc/html/rfc5905): Network Time Protocol Version 4: Protocol and Algorithms Specification
- [Wikipedia](https://en.wikipedia.org/wiki/EtherType#Overview): Overview over IEEE 802.3 and EtherType
