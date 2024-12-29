use std::env;
use std::io::{self, Write};
use std::net::{IpAddr, TcpStream};
use std::str::FromStr;
use std::process;
use std::sync::mpsc::{Sender, channel};
use std::thread;

const MAX: u16 = 65535;

struct Arguments {
    flag: String,
    ipaddr: IpAddr,
    threads: u16,
}

impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        if args.len() < 2 {
            return Err("ERROR 0x01: Insufficient arguments provided.");
        } else if args.len() > 4 {
            return Err("ERROR 0x02: Excessive argument(s) provided.");
        }

        let f = args[1].clone();

        if let Ok(ipaddr) = IpAddr::from_str(&f) {
            return Ok(Arguments {
                flag: String::from(""),
                ipaddr,
                threads: 4,
            });
        } else {
            let flag = args[1].clone();
            if flag.contains("-h") || flag.contains("-help") && args.len() == 2 {
                println!("Usage: ip_sniffer <flags> <ipaddr> \r\n-j to select thread number \r\n-h or -help to show this help message");
                return Err("ERROR 0x00: Help");
            } else if flag.contains("-h") || flag.contains("-help") {
                return Err("ERROR 0x03: The help flag is exclusive");
            } else if flag.contains("-j") {
                let ipaddr = match IpAddr::from_str(&args[3]) {
                    Ok(s) => s,
                    Err(_) => return Err("ERROR 0x04: Invalid IPADDR; must be IPv4 or IPv6"),
                };
                let threads = match args[2].parse::<u16>() {
                    Ok(s) => s,
                    Err(_) => return Err("ERROR 0x05: Failed to parse thread number"),
                };
                return Ok(Arguments {
                    threads,
                    flag,
                    ipaddr,
                });
            } else {
                return Err("ERROR 0x06: Invalid Syntax");
            }
        }
    }
}

fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr, num_threads: u16) {
    let mut port: u16 = start_port + 1;
    loop {
        match TcpStream::connect((addr, port)) {
            Ok(_) => {
                print!(".");
                io::stdout().flush().unwrap();
                tx.send(port).unwrap();
            }
            Err(_) => {
            }
        }

        if (MAX - port) <= num_threads {
            break;
        }
        port += num_threads;
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let arguments = Arguments::new(&args).unwrap_or_else(
        |err| {
            if err.contains("ERROR 0x00: Help") {
                process::exit(0);
            } else {
                eprintln!("{} exited with {}", program, err);
                process::exit(0);
            }
        }
    );

    let num_threads = arguments.threads;
    let flag = arguments.flag;
    let addr = arguments.ipaddr;
    let (tx, rx) = channel();
    
    println!("{} is running with {} flag set to {} on {}", program, flag, num_threads, addr); 

    for i in 0..num_threads {
        let tx = tx.clone();
        thread::spawn(move || {
            scan(tx, i, addr, num_threads);
        });
    }

    println!("\nScan Complete.");

    let mut out = vec![];
    drop(tx);
    for p in rx {
        out.push(p);
    }

    println!("");
    out.sort();
    for v in out {
        println!("{} is open", v);
    }
}
