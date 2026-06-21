use pcap::{Capture, Device, Error as PcapError, Linktype};
use std::net::Ipv4Addr;

const DEFAULT_COUNT: usize = 20;
const IDLE_TIMEOUTS_BEFORE_STOP: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProtocolFilter {
    Any,
    Tcp,
    Udp,
    Icmp,
}

#[derive(Debug)]
struct SniffOptions {
    interface: Option<String>,
    protocol: ProtocolFilter,
    port: Option<u16>,
    count: usize,
    promisc: bool,
}

impl Default for SniffOptions {
    fn default() -> Self {
        Self {
            interface: None,
            protocol: ProtocolFilter::Any,
            port: None,
            count: DEFAULT_COUNT,
            promisc: true,
        }
    }
}

#[derive(Debug, Default)]
struct CaptureStats {
    shown: usize,
    decoded: usize,
    skipped: usize,
    tcp: usize,
    udp: usize,
    icmp: usize,
    other: usize,
}

#[derive(Debug, PartialEq, Eq)]
struct PacketInfo {
    src_ip: Ipv4Addr,
    dst_ip: Ipv4Addr,
    src_port: Option<u16>,
    dst_port: Option<u16>,
    protocol: String,
    length: usize,
}

impl PacketInfo {
    fn pretty(&self) -> String {
        let src = endpoint(self.src_ip, self.src_port);
        let dst = endpoint(self.dst_ip, self.dst_port);

        format!(
            "[{}] {} -> {} ({} bytes)",
            self.protocol, src, dst, self.length
        )
    }
}

pub fn run(input: &str) {
    let parts: Vec<&str> = input.split_whitespace().collect();

    match parse_args(&parts) {
        Ok(CommandMode::Help) => print_usage(),
        Ok(CommandMode::List) => list_interfaces(),
        Ok(CommandMode::Capture(options)) => capture_packets(options),
        Err(e) => {
            eprintln!("\x1b[91m{}\x1b[0m", e);
            print_usage();
        }
    }
}

enum CommandMode {
    Help,
    List,
    Capture(SniffOptions),
}

fn parse_args(parts: &[&str]) -> Result<CommandMode, String> {
    let mut options = SniffOptions::default();
    let mut i = 1;

    while i < parts.len() {
        match parts[i] {
            "--help" | "-h" => return Ok(CommandMode::Help),
            "--list" => return Ok(CommandMode::List),
            "--interface" | "-i" => {
                i += 1;
                let Some(value) = parts.get(i) else {
                    return Err("Missing interface name after --interface".into());
                };
                options.interface = Some((*value).to_string());
            }
            "--tcp" => options.protocol = set_protocol(options.protocol, ProtocolFilter::Tcp)?,
            "--udp" => options.protocol = set_protocol(options.protocol, ProtocolFilter::Udp)?,
            "--icmp" => options.protocol = set_protocol(options.protocol, ProtocolFilter::Icmp)?,
            "--port" | "-p" => {
                i += 1;
                let Some(value) = parts.get(i) else {
                    return Err("Missing port number after --port".into());
                };
                options.port = Some(
                    value
                        .parse::<u16>()
                        .map_err(|_| format!("Invalid port '{}'", value))?,
                );
            }
            "--count" | "-c" => {
                i += 1;
                let Some(value) = parts.get(i) else {
                    return Err("Missing packet count after --count".into());
                };
                options.count = value
                    .parse::<usize>()
                    .map_err(|_| format!("Invalid count '{}'", value))?;
                if options.count == 0 {
                    return Err("Count must be greater than zero".into());
                }
            }
            "--no-promisc" => options.promisc = false,
            other => return Err(format!("Unknown sniff option '{}'", other)),
        }

        i += 1;
    }

    if options.port.is_some() && options.protocol == ProtocolFilter::Icmp {
        return Err("ICMP packets do not have TCP/UDP ports".into());
    }

    Ok(CommandMode::Capture(options))
}

fn set_protocol(current: ProtocolFilter, next: ProtocolFilter) -> Result<ProtocolFilter, String> {
    if current != ProtocolFilter::Any && current != next {
        return Err("Use only one protocol filter: --tcp, --udp, or --icmp".into());
    }

    Ok(next)
}

fn list_interfaces() {
    match Device::list() {
        Ok(devices) => {
            println!("Available interfaces:");
            for dev in devices {
                let desc = dev.desc.unwrap_or_else(|| "no description".into());
                let state = if dev.flags.is_running() {
                    "running"
                } else {
                    "down"
                };
                println!("  {:<12} {:<8} {}", dev.name, state, desc);
            }
        }
        Err(e) => eprintln!("Failed to list interfaces: {}", e),
    }
}

fn capture_packets(options: SniffOptions) {
    let iface = match options.interface.clone() {
        Some(interface) => interface,
        None => match Device::lookup() {
            Ok(Some(dev)) => dev.name,
            Ok(None) => {
                eprintln!("No default interface found. Use: sniff --list");
                return;
            }
            Err(e) => {
                eprintln!("Failed to lookup default interface: {}", e);
                return;
            }
        },
    };

    let filter = build_bpf_filter(&options);

    println!(
        "\x1b[93mPanda sniffing on {}\x1b[0m (filter: {}, count: {}, promisc: {})",
        iface,
        filter.as_deref().unwrap_or("ipv4"),
        options.count,
        options.promisc
    );

    let inactive = match Capture::from_device(iface.as_str()) {
        Ok(dev) => dev,
        Err(e) => {
            eprintln!("Failed to find interface '{}': {}", iface, e);
            eprintln!("Run `sniff --list` to see valid interfaces.");
            return;
        }
    };

    let mut cap = match inactive
        .promisc(options.promisc)
        .snaplen(65535)
        .timeout(1000)
        .open()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to open capture on '{}': {}", iface, e);
            eprintln!("Packet capture usually needs elevated privileges.");
            eprintln!("Try running Panda with sudo, or pick another interface with sniff --list.");
            return;
        }
    };

    if let Some(filter) = &filter {
        if let Err(e) = cap.filter(filter, true) {
            eprintln!("Failed to apply capture filter '{}': {}", filter, e);
            return;
        }
    }

    let linktype = cap.get_datalink();
    let link_name = linktype
        .get_name()
        .unwrap_or_else(|_| format!("linktype {}", linktype.0));
    println!("Link type: {}", link_name);

    let mut stats = CaptureStats::default();
    let mut idle_timeouts = 0usize;

    while stats.shown < options.count {
        match cap.next_packet() {
            Ok(packet) => {
                idle_timeouts = 0;
                match decode_packet(packet.data, packet.data.len(), linktype) {
                    Some(info) => {
                        stats.decoded += 1;
                        record_protocol(&mut stats, &info.protocol);

                        if matches_filters(&info, &options) {
                            println!("{}", info.pretty());
                            stats.shown += 1;
                        } else {
                            stats.skipped += 1;
                        }
                    }
                    None => stats.skipped += 1,
                }
            }
            Err(PcapError::TimeoutExpired) => {
                idle_timeouts += 1;
                if idle_timeouts >= IDLE_TIMEOUTS_BEFORE_STOP {
                    eprintln!("No packets matched before timeout.");
                    break;
                }
            }
            Err(PcapError::NoMorePackets) => break,
            Err(e) => {
                eprintln!("Capture failed: {}", e);
                break;
            }
        }
    }

    println!(
        "\x1b[92mPanda sniff finished.\x1b[0m shown: {}, decoded: {}, skipped: {}, tcp: {}, udp: {}, icmp: {}, other: {}",
        stats.shown, stats.decoded, stats.skipped, stats.tcp, stats.udp, stats.icmp, stats.other
    );
}

fn print_usage() {
    println!("Usage:");
    println!("  sniff --list");
    println!(
        "  sniff [--interface IFACE] [--tcp|--udp|--icmp] [--port N] [--count N] [--no-promisc]"
    );
    println!();
    println!("Examples:");
    println!("  sniff --list");
    println!("  sniff --interface en0 --count 10");
    println!("  sniff --tcp --port 443 --count 20");
    println!("  sniff --icmp --count 5");
}

fn build_bpf_filter(options: &SniffOptions) -> Option<String> {
    let mut parts = vec!["ip".to_string()];

    match options.protocol {
        ProtocolFilter::Any => {}
        ProtocolFilter::Tcp => parts.push("tcp".into()),
        ProtocolFilter::Udp => parts.push("udp".into()),
        ProtocolFilter::Icmp => parts.push("icmp".into()),
    }

    if let Some(port) = options.port {
        parts.push(format!("port {}", port));
    }

    Some(parts.join(" and "))
}

fn decode_packet(data: &[u8], length: usize, linktype: Linktype) -> Option<PacketInfo> {
    let (ip_start, ethertype) = ipv4_start(data, linktype)?;

    if ethertype != 0x0800 {
        return None;
    }

    decode_ipv4_packet(&data[ip_start..], length)
}

fn ipv4_start(data: &[u8], linktype: Linktype) -> Option<(usize, u16)> {
    match linktype {
        Linktype::ETHERNET => ethernet_ipv4_start(data),
        Linktype::LINUX_SLL => linux_sll_ipv4_start(data),
        Linktype::RAW | Linktype::IPV4 => Some((0, 0x0800)),
        Linktype::NULL | Linktype::LOOP => loopback_ipv4_start(data),
        _ if data.first().map(|b| b >> 4) == Some(4) => Some((0, 0x0800)),
        _ => None,
    }
}

fn ethernet_ipv4_start(data: &[u8]) -> Option<(usize, u16)> {
    if data.len() < 14 {
        return None;
    }

    let mut offset = 14;
    let mut ethertype = u16::from_be_bytes([data[12], data[13]]);

    while matches!(ethertype, 0x8100 | 0x88a8 | 0x9100) {
        if data.len() < offset + 4 {
            return None;
        }
        ethertype = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);
        offset += 4;
    }

    Some((offset, ethertype))
}

fn linux_sll_ipv4_start(data: &[u8]) -> Option<(usize, u16)> {
    if data.len() < 16 {
        return None;
    }

    Some((16, u16::from_be_bytes([data[14], data[15]])))
}

fn loopback_ipv4_start(data: &[u8]) -> Option<(usize, u16)> {
    if data.len() < 4 {
        return None;
    }

    let family_le = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let family_be = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);

    if family_le == 2 || family_be == 2 || family_le == 24 || family_be == 24 {
        Some((4, 0x0800))
    } else {
        None
    }
}

fn decode_ipv4_packet(data: &[u8], length: usize) -> Option<PacketInfo> {
    if data.len() < 20 || data[0] >> 4 != 4 {
        return None;
    }

    let ihl = (data[0] & 0x0f) as usize * 4;
    if ihl < 20 || data.len() < ihl {
        return None;
    }

    let protocol_number = data[9];
    let src_ip = Ipv4Addr::new(data[12], data[13], data[14], data[15]);
    let dst_ip = Ipv4Addr::new(data[16], data[17], data[18], data[19]);
    let transport = &data[ihl..];

    let (protocol, src_port, dst_port) = match protocol_number {
        1 => ("ICMP".to_string(), None, None),
        6 => {
            if transport.len() < 20 {
                return None;
            }
            (
                "TCP".to_string(),
                read_port(transport, 0),
                read_port(transport, 2),
            )
        }
        17 => {
            if transport.len() < 8 {
                return None;
            }
            (
                "UDP".to_string(),
                read_port(transport, 0),
                read_port(transport, 2),
            )
        }
        other => (format!("IPv4({})", other), None, None),
    };

    Some(PacketInfo {
        src_ip,
        dst_ip,
        src_port,
        dst_port,
        protocol,
        length,
    })
}

fn read_port(data: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_be_bytes([
        *data.get(offset)?,
        *data.get(offset + 1)?,
    ]))
}

fn endpoint(ip: Ipv4Addr, port: Option<u16>) -> String {
    match port {
        Some(port) => format!("{}:{}", ip, port),
        None => ip.to_string(),
    }
}

fn matches_filters(info: &PacketInfo, options: &SniffOptions) -> bool {
    let protocol_matches = match options.protocol {
        ProtocolFilter::Any => true,
        ProtocolFilter::Tcp => info.protocol == "TCP",
        ProtocolFilter::Udp => info.protocol == "UDP",
        ProtocolFilter::Icmp => info.protocol == "ICMP",
    };

    let port_matches = options
        .port
        .map(|port| info.src_port == Some(port) || info.dst_port == Some(port))
        .unwrap_or(true);

    protocol_matches && port_matches
}

fn record_protocol(stats: &mut CaptureStats, protocol: &str) {
    match protocol {
        "TCP" => stats.tcp += 1,
        "UDP" => stats.udp += 1,
        "ICMP" => stats.icmp += 1,
        _ => stats.other += 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_ethernet_ipv4_tcp_packet() {
        let mut packet = ethernet_header(0x0800);
        packet.extend(ipv4_header(6, [192, 168, 1, 10], [93, 184, 216, 34], 20));
        packet.extend([0x1f, 0x90, 0x01, 0xbb]);
        packet.extend([0; 16]);

        let info = decode_packet(&packet, packet.len(), Linktype::ETHERNET).unwrap();

        assert_eq!(info.protocol, "TCP");
        assert_eq!(info.src_ip, Ipv4Addr::new(192, 168, 1, 10));
        assert_eq!(info.dst_ip, Ipv4Addr::new(93, 184, 216, 34));
        assert_eq!(info.src_port, Some(8080));
        assert_eq!(info.dst_port, Some(443));
    }

    #[test]
    fn decodes_linux_sll_udp_packet() {
        let mut packet = vec![0; 14];
        packet.extend([0x08, 0x00]);
        packet.extend(ipv4_header(17, [10, 0, 0, 1], [8, 8, 8, 8], 8));
        packet.extend([0xd9, 0x03, 0x00, 0x35, 0, 8, 0, 0]);

        let info = decode_packet(&packet, packet.len(), Linktype::LINUX_SLL).unwrap();

        assert_eq!(info.protocol, "UDP");
        assert_eq!(info.src_port, Some(55555));
        assert_eq!(info.dst_port, Some(53));
    }

    #[test]
    fn validates_conflicting_protocol_filters() {
        let parts = ["sniff", "--tcp", "--udp"];
        assert!(parse_args(&parts).is_err());
    }

    fn ethernet_header(ethertype: u16) -> Vec<u8> {
        let mut header = vec![0; 12];
        header.extend(ethertype.to_be_bytes());
        header
    }

    fn ipv4_header(protocol: u8, src: [u8; 4], dst: [u8; 4], transport_len: u16) -> Vec<u8> {
        let total_len = 20 + transport_len;
        let mut header = vec![0; 20];
        header[0] = 0x45;
        header[2..4].copy_from_slice(&total_len.to_be_bytes());
        header[8] = 64;
        header[9] = protocol;
        header[12..16].copy_from_slice(&src);
        header[16..20].copy_from_slice(&dst);
        header
    }
}
