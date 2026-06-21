use pcap::{Capture, Device};
use std::net::Ipv4Addr;

#[derive(Debug)]
struct PacketInfo {
    src_ip: Option<Ipv4Addr>,
    dst_ip: Option<Ipv4Addr>,
    src_port: Option<u16>,
    dst_port: Option<u16>,
    protocol: &'static str,
    length: usize,
}

impl PacketInfo {
    fn pretty(&self) -> String {
        let src_ip = self
            .src_ip
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| "?".into());
        let dst_ip = self
            .dst_ip
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| "?".into());
        let src_port = self
            .src_port
            .map(|p| p.to_string())
            .unwrap_or_else(|| "?".into());
        let dst_port = self
            .dst_port
            .map(|p| p.to_string())
            .unwrap_or_else(|| "?".into());

        format!(
            "[{}] {}:{} → {}:{} ({} bytes)",
            self.protocol, src_ip, src_port, dst_ip, dst_port, self.length
        )
    }
}

pub fn run(input: &str) {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.is_empty() || parts[0] != "sniff" {
        eprintln!("Usage: sniff [--interface IFACE] [--tcp|--udp] [--port N] [--count N]");
        return;
    }

    let mut interface: Option<String> = None;
    let mut filter_tcp = false;
    let mut filter_udp = false;
    let mut filter_port: Option<u16> = None;
    let mut count: Option<usize> = None;

    let mut i = 1;
    while i < parts.len() {
        match parts[i] {
            "--interface" => {
                if i + 1 < parts.len() {
                    interface = Some(parts[i + 1].to_string());
                    i += 1;
                } else {
                    eprintln!("--interface requires a value");
                    return;
                }
            }
            "--tcp" => filter_tcp = true,
            "--udp" => filter_udp = true,
            "--port" => {
                if i + 1 < parts.len() {
                    match parts[i + 1].parse::<u16>() {
                        Ok(p) => {
                            filter_port = Some(p);
                            i += 1;
                        }
                        Err(_) => {
                            eprintln!("Invalid port value");
                            return;
                        }
                    }
                } else {
                    eprintln!("--port requires a value");
                    return;
                }
            }
            "--count" => {
                if i + 1 < parts.len() {
                    match parts[i + 1].parse::<usize>() {
                        Ok(c) => {
                            count = Some(c);
                            i += 1;
                        }
                        Err(_) => {
                            eprintln!("Invalid count value");
                            return;
                        }
                    }
                } else {
                    eprintln!("--count requires a value");
                    return;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let iface_name = match interface {
        Some(name) => name,
        None => {
            // pick default device
            match Device::lookup() {
                Ok(dev) => dev.name,
                Err(_) => {
                    eprintln!("Failed to find default interface. Use --interface IFACE.");
                    return;
                }
            }
        }
    };

    println!(
        "\x1b[93mPanda sniffing on interface {}\x1b[0m (tcp: {}, udp: {}, port: {:?}, count: {:?})",
        iface_name, filter_tcp, filter_udp, filter_port, count
    );

    let mut cap = match Capture::from_device(iface_name.as_str())
        .and_then(|d| d.promisc(true).snaplen(65535).open())
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to open capture: {}", e);
            return;
        }
    };

    let mut seen = 0usize;

    while let Ok(packet) = cap.next() {
        let data = packet.data;
        let length = data.len();

        if let Some(info) = decode_packet(data, length) {
            if filter_tcp && info.protocol != "TCP" {
                continue;
            }
            if filter_udp && info.protocol != "UDP" {
                continue;
            }
            if let Some(p) = filter_port {
                if info.src_port != Some(p) && info.dst_port != Some(p) {
                    continue;
                }
            }

            println!("{}", info.pretty());
            seen += 1;

            if let Some(limit) = count {
                if seen >= limit {
                    break;
                }
            }
        }
    }

    println!("\x1b[92mPanda sniff finished. Captured {} packets.\x1b[0m", seen);
}

fn decode_packet(data: &[u8], length: usize) -> Option(PacketInfo) {
    // Ethernet header: 14 bytes
    if data.len() < 14 {
        return None;
    }

    let ethertype = u16::from_be_bytes([data[12], data[13]]);
    if ethertype != 0x0800 {
        // not IPv4
        return None;
    }

    // IPv4 header starts at offset 14
    if data.len() < 14 + 20 {
        return None;
    }

    let ip_header_start = 14;
    let version_ihl = data[ip_header_start];
    let ihl = (version_ihl & 0x0f) as usize * 4;
    if data.len() < ip_header_start + ihl {
        return None;
    }

    let protocol = data[ip_header_start + 9];
    let src_ip = Ipv4Addr::new(
        data[ip_header_start + 12],
        data[ip_header_start + 13],
        data[ip_header_start + 14],
        data[ip_header_start + 15],
    );
    let dst_ip = Ipv4Addr::new(
        data[ip_header_start + 16],
        data[ip_header_start + 17],
        data[ip_header_start + 18],
        data[ip_header_start + 19],
    );

    let transport_start = ip_header_start + ihl;
    if data.len() < transport_start + 4 {
        return None;
    }

    let (proto_name, src_port, dst_port) = match protocol {
        6 => {
            // TCP
            if data.len() < transport_start + 20 {
                return None;
            }
            let src_port = u16::from_be_bytes([data[transport_start], data[transport_start + 1]]);
            let dst_port = u16::from_be_bytes([data[transport_start + 2], data[transport_start + 3]]);
            ("TCP", Some(src_port), Some(dst_port))
        }
        17 => {
            // UDP
            if data.len() < transport_start + 8 {
                return None;
            }
            let src_port = u16::from_be_bytes([data[transport_start], data[transport_start + 1]]);
            let dst_port = u16::from_be_bytes([data[transport_start + 2], data[transport_start + 3]]);
            ("UDP", Some(src_port), Some(dst_port))
        }
        _ => ("OTHER", None, None),
    };

    Some(PacketInfo {
        src_ip: Some(src_ip),
        dst_ip: Some(dst_ip),
        src_port,
        dst_port,
        protocol: proto_name,
        length,
    })
}
