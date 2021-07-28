use std::net::IpAddr;
use std::net::ToSocketAddrs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use clap::{self, value_t_or_exit, Arg};
use colored::{self, Colorize};
use pretty_ping::Pinger;
use rand::random;

// Package meta info
const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

struct Arguments {
    to_hostname: String,
    to_addr: IpAddr,
    count: u16,
    interval: u64,
    timeout: u64,
    size: usize,
}

fn parse_cmd() -> Arguments {
    let matches = clap::App::new(NAME)
        .version(VERSION)
        .author(AUTHOR)
        .about(DESCRIPTION)
        .arg(
            Arg::with_name("hostname")
                .value_name("HOSTNAME")
                .help("domain name or ip address")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .value_name("COUNT")
                .help("stop after <count> replies")
                .default_value("0")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("interval")
                .short("i")
                .long("interval")
                .value_name("INTERVAL")
                .help("millisecond between sending each packet")
                .default_value("1000")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("timeout")
                .short("t")
                .long("timeout")
                .value_name("TIMEOUT")
                .help("millisecond to wait for response")
                .default_value("1000")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("size")
            .short("s")
            .long("packetsize")
            .value_name("SIZE")
            .help("specify the number of data bytes to be sent.  The default is 56, which translates into 64 ICMP data bytes when combined with the 8 bytes of ICMP header data.")
            .default_value("56")
            .takes_value(true),
        )
        .get_matches();

    let to_hostname = value_t_or_exit!(matches.value_of("hostname"), String);
    let count = value_t_or_exit!(matches.value_of("count"), u16);
    let interval = value_t_or_exit!(matches.value_of("interval"), u64);
    let timeout = value_t_or_exit!(matches.value_of("timeout"), u64);
    let size = value_t_or_exit!(matches.value_of("size"), usize);

    let to_addr = match to_hostname.parse::<IpAddr>() {
        Ok(address) => Some(address),
        Err(_) => match (to_hostname.clone(), 0).to_socket_addrs() {
            Ok(mut resolve_result) => {
                if let Some(resolve) = resolve_result.next() {
                    Some(resolve.ip())
                } else {
                    println!("no ip from the DNS resolver.");
                    None
                }
            }
            Err(e) => {
                println!("failed to resolve hostname {}.", e);
                None
            }
        },
    };

    if to_addr.is_none() {
        std::process::exit(1)
    }

    return Arguments {
        to_hostname,
        to_addr: to_addr.unwrap(),
        count,
        interval,
        timeout,
        size,
    };
}

fn main() {
    let arguments = parse_cmd();

    let interval = Duration::from_millis(arguments.interval);
    let timeout = Duration::from_millis(arguments.timeout);

    let pinger = match Pinger::new(arguments.to_addr) {
        Ok(res) => res,
        Err(e) => panic!("{}", e),
    };

    println!(
        "PING {} ({}): {} data bytes",
        format!("{}", arguments.to_hostname).blue(),
        format!("{}", arguments.to_addr).blue(),
        arguments.size
    );

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let mut cur_count = 0;
    let ident = random::<u16>();

    let mut transmitted = 0;
    let mut received = 0;
    let mut min = Duration::new(0, 0);
    let mut max = Duration::new(0, 0);
    let avg;
    let mut sum = 0.0;
    let mut square_sum = 0.0;
    let stddev;

    while running.load(Ordering::SeqCst) {
        // if count == 0, we start a endless ping unless get a KeyboardInterrupt from user.
        if arguments.count != 0 && cur_count >= arguments.count {
            break;
        }
        cur_count += 1;

        let start_at = Instant::now();
        let res = pinger.ping(ident, cur_count, arguments.size, timeout); // use the cur_count as ping sequence number.
        let elapsed_ms = Instant::now() - start_at;

        transmitted += 1;
        match res {
            Ok((reply, duration)) => {
                received += 1;
                let cost_time = duration.as_secs_f64() * 1000 as f64; // as_millis has precision problem

                if min == Duration::new(0, 0) || duration < min {
                    min = duration;
                }
                if duration > max {
                    max = duration;
                }
                sum += duration.as_secs_f64() * 1000f64;
                square_sum += duration.as_secs_f64() * 1000f64 * duration.as_secs_f64() * 1000f64;

                println!(
                    "{} bytes from {}: icmp_seq={} ttl={} time={} ms",
                    reply.size,
                    format!("{}", reply.source).blue(),
                    reply.seq,
                    match reply.ttl {
                        Some(ttl) => format!("{}", ttl),
                        None => "?".to_string(),
                    },
                    colorful_rtt(cost_time)
                );
            }
            Err(e) => {
                println!("{}", format!("{}", e).red());
            }
        };

        if elapsed_ms < interval {
            thread::sleep(interval - elapsed_ms);
        }
    }

    // echo the statistics
    println!("\n--- {} ping statistics ---", arguments.to_hostname.blue());
    let loss = if transmitted == 0 {
        colored::ColoredString::from("0.0")
    } else {
        let v = ((transmitted - received) as f64 / transmitted as f64) * 100.0;
        if v > 0.0 {
            format!("{:.1}", v).red()
        } else {
            colored::ColoredString::from(format!("{:.1}", v).as_str())
        }
    };

    avg = sum / received as f64;

    stddev = ((square_sum / received as f64) - avg * avg).sqrt();
    println!(
        "{} packets transmitted, {} packets received, {:.1}% packet loss",
        format!("{}", transmitted).magenta(),
        format!("{}", received).magenta(),
        loss
    );

    println!(
        "round-trip min/avg/max/stddev = {}/{}/{}/{} ms",
        colorful_rtt(min.as_secs_f64() * 1000f64),
        colorful_rtt(avg),
        colorful_rtt(max.as_secs_f64() * 1000f64),
        colorful_rtt(stddev), // this always greee...
    );
}

fn colorful_rtt(time: f64) -> colored::ColoredString {
    if 0f64 < time && time <= 100f64 {
        format!("{:.3}", time).green()
    } else if 100f64 < time && time < 200f64 {
        format!("{:.3}", time).yellow()
    } else {
        format!("{:.3}", time).red()
    }
}
