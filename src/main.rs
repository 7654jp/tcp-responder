use std::error::Error;
use std::io::{self, BufRead, Read, Write};
use std::net::{TcpListener, TcpStream};

#[inline]
fn print_char(c: &u8) {
    print!(
        "{}",
        if c.is_ascii_graphic() && *c != 0x7f {
            *c as char
        } else {
            '.'
        }
    );
}

fn hex_print(hex: &[u8], validlen: usize) {
    if hex.len() > 65535 {
        eprintln!("Array with more than 65535 elements isn't supported");
        std::process::exit(1);
    }
    let mut bslice = [0_u8; 8];
    for (index, cc) in hex[..validlen].iter().enumerate() {
        bslice[index % 8] = *cc;
        print!("{:02x} ", cc);
        if index % 16 == 0 || index + 1 == validlen {
            print!(" | ");
            for pc in bslice.iter() {
                print_char(pc);
            }
            println!();
        } else if index % 8 == 0 {
            print!("- ");
        }
    }
}

fn respond(stream: &mut TcpStream, print_tcp: bool) -> Result<(), Box<dyn Error>> {
    let mut buf = [0_u8; 65535];
    loop {
        let read = stream.read(&mut buf);
        match read {
            Ok(0) => break,
            Ok(n) => {
                println!("\n=====\n\nSocket Input:\n");
                // Print hex if requested
                if print_tcp {
                    hex_print(&buf, n);
                    println!("\n=====\n");
                }

                // Print buffer in nearly-readable form
                for cc in &buf[..n] {
                    if *cc == 0x20_u8 {
                        print!(" ");
                    } else {
                        print_char(cc);
                    }
                }

                println!("\n\n=====\n");
                println!(
                    "Please send the HTTP response back:\nType '?REVERT?' at new line to undo the last line.\nType '?END?' at new line to send.\n"
                );

                // Response writer
                let stdin = io::stdin();
                let mut line = String::new();
                let mut lines: Vec<String> = Vec::new();
                loop {
                    line.clear();
                    if stdin.lock().read_line(&mut line).unwrap() == 0 {
                        break;
                    }

                    let trimmed = line.trim().to_string();
                    if trimmed == "?END?" {
                        break;
                    } else if trimmed == "?REVERT?" {
                        lines.pop();
                        println!("\nLine reverted. Now:");
                        for line in &lines {
                            println!("{}", line);
                        }
                    } else {
                        lines.push(trimmed);
                    }
                }

                let mut send_buffer: Vec<u8> = Vec::new();
                for line in &lines {
                    send_buffer.extend_from_slice(line.as_bytes());
                    send_buffer.extend_from_slice(b"\n");
                }
                send_buffer.extend_from_slice(b"\n");
                stream.write_all(&send_buffer)?;
                println!("\nResponse sent!");
            }
            Err(e) => {
                eprintln!("{:?}", e);
                return Err(e.into());
            }
        }
    }
    Ok(())
}

fn main() {
    // Get arguments
    let args: Vec<String> = std::env::args().collect();
    let mut param: (u16, bool) = (0, false);
    let mut wrong_parameter = false;

    let mut p0: String = String::new();
    let mut p1: String = String::new();
    if args.len() != 3 {
        let stdin = io::stdin();
        println!("Parameter format is wrong!\nPlease re-enter the values.\n");
        print!("Port number [u16]: ");
        io::stdout().flush().unwrap();
        stdin.lock().read_line(&mut p0).unwrap();
        print!("Output TCP stream in hexadecimal? [t/f]: ");
        io::stdout().flush().unwrap();
        stdin.lock().read_line(&mut p1).unwrap();
        p0 = p0.trim().to_string();
        p1 = p1.trim().to_string();
        wrong_parameter = true;
    } else {
        p0 = args[1].clone();
        p1 = args[2].clone();
    }
    param.0 = p0.parse::<u16>().unwrap_or_else(|_err| {
        println!("Usage: ./tcp_responder [port number] [t/f :Output TCP stream in hexadecimal]");
        println!("Couldn't get the valid value from 'port number'");
        println!("Using default value: port 30000");
        wrong_parameter = true;
        p0 = 30000.to_string();
        30000
    });
    param.1 = if p1 == "t" {
        true
    } else if p1 == "f" {
        false
    } else {
        println!("Usage: ./tcp_responder [port number] [t/f :Output TCP stream in hexadecimal]");
        println!("Couldn't get the valid value from 'Output TCP stream in hexadecimal'");
        println!("Using default value: false");
        wrong_parameter = true;
        p1 = "f".to_string();
        false
    };

    if wrong_parameter {
        println!(
            "\nYou can start the program with same values by:\n./tcp_responder {} {}\n",
            p0, p1
        );
    }

    // Getting ready for listening to the port
    let addr = format!("localhost:{}", param.0);
    let listener = TcpListener::bind(addr.clone()).unwrap();

    // Ready
    println!("Listening at {}", addr);
    loop {
        let (mut stream, sockaddr) = listener.accept().unwrap();
        println!("[DEBUG] Socket address: {:?}", sockaddr);
        std::thread::spawn(move || {
            respond(&mut stream, param.1).unwrap();
        });
    }
}
