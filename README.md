# raw-networking-demo

> A collection of OSI/ISO layer implementations for crafting and sending raw Ethernet/IP packets to query real-world services.

This repository contains a collection of Rust programs, each demonstrating the use of different ISO/OSI layer stacks over Ethernet to query real-world services. The programs operate using raw Linux sockets, offering an insightful glimpse into the implementation of various networking stacks and how networking is done at a low level,
giving an insight over what the Operating System does when you *simply request XY from the internet*.

## Features

- Well-structured and documented implementations of various ISO/OSI layer stacks.
- Demonstrates networking stacks using raw Linux sockets.
- Each demo focuses on specific transport, internet and application layer protocol combinations.
- Query real-world services give a demonstration of it's power.

## Project Structure

Each demo is implemented as a separate Rust crate, following the naming convention:

```
demo-<applciation protocol>-<transport-protocol>-<internet protocol>
```

Each create comes with rich explainations and documentations about the underlying concepts and workings, on which the networking stack operates.

## Installation and Usage

### Prerequisites

- Rust installation
- Cargo installation (with Rust)
- Running on a linux-based distribution
- Bash shell
- iptables

### Installation

Clone this repository from [GitHub](https://github.com/RossAdrian/raw-networking-demo):

```bash
git clone https://github.com/RossAdrian/raw-networking-demo.git
```

### Running the Demos

Execute a demo with the following command:

```bash
cargo run --release <crate> -- <args>
```

For commandline arguments, reffer to the corresponding crate README, or run with `--help` to get the help message for this crate.

### Debugging

Best way to debug, or simply observe the packet flow, is observing the packet flow in [Wireshark](https://www.wireshark.org/).

## Intended Purpose

The primary goal of this project is to provide:

1. A practical understanding of networking through code.
2. Insights into ISO/OSI layer implementations for various networking stacks.

This is **NOT** a introduction into how to implement your *Rust* networking software. It is not recommended reinvent the wheel for production software.
Your operating system knows what it does, and does it *(normally ...)* well. This is just a insight about what the operating system does when you sent
a request to *XY*, while you as a programmer normally stand on the application layer, selecting a *UDP* of *TCP* socket.

## License

MIT © [Adrian Roß](https://github.com/RossAdrian)
