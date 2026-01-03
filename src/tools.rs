#[inline]
pub fn print_char(c: &u8) {
    print!(
        "{}",
        if c.is_ascii_graphic() && *c != 0x7f {
            *c as char
        } else {
            '.'
        }
    );
}

pub fn hex_print(hex: &[u8], validlen: usize, printchar: bool) {
    let mut bslice = [0_u8; 8];
    for (index, cc) in hex[..validlen].iter().enumerate() {
        bslice[index % 8] = *cc;
        print!("{:02x} ", cc);
        if index % 16 == 0 || index + 1 == validlen {
            if printchar {
                print!(" | ");
                for pc in bslice.iter() {
                    print_char(pc);
                }
            }
            println!();
        } else if index % 8 == 0 {
            print!("- ");
        }
    }
}