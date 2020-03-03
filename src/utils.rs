use crate::errors::ParseError;
use std::error;

pub fn byte_combine(a: u8, b: u8) -> u16 {
    ((a as u16) << 8) | b as u16
}

pub fn bytes_to_name_offset(a: u8, b: u8) -> usize {
    (byte_combine(a, b) & 0b11_1111_1111_1111) as usize
}

pub fn extract_name(
    bytes: &[u8],
    mut offset: usize,
) -> Result<(Vec<String>, usize), &dyn error::Error> {
    let mut name = Vec::new();
    loop {
        let label_len = bytes[offset] as usize;

        if label_len == 0 {
            offset += 1;
            break;
        }
        if (label_len >> 6) == 3 {
            let name_offset = bytes_to_name_offset(bytes[offset], bytes[offset + 1]);
            if name_offset > offset {
                return Err(&ParseError::PointerForward);
            }
            let (mut ref_name, _loffset) = extract_name(bytes, name_offset)?;
            name.append(&mut ref_name);
            break;
        }
        if label_len > 63 {
            panic!("omg {:b}", label_len >> 6);
        }

        let label_start = 1 + offset;
        let label_end = label_start + label_len;
        let label = match std::str::from_utf8(&bytes[label_start..label_end]) {
            Ok(v) => v.to_string(),
            Err(e) => {
                panic!("invalid utf8 label {}; label_len {}", e, label_len);
            }
        };
        name.push(label);
        offset += 1 + label_len;
    }
    Ok((name, offset))
}

fn read_dec(input: &str, offset: usize) -> Result<u8, std::num::ParseIntError> {
    let n: u8 = input[offset+1..offset+4].parse()?;
    Ok(n)
}

// WIP: passes two initial test cases but doesn't handle bounds checks or
// all of the special cases in rfc4343
pub fn rfc4343_to_wire(input: &str) -> Vec<Vec<u8>> {
    let mut labels = Vec::new();
    let mut cur: Vec<u8> = Vec::new();
    let input_bytes: Vec<u8> = input.bytes().collect();

    let mut i = 0;
    while i < input_bytes.len() {
        let b = input_bytes[i];
        if b == 0x5c { // \
            // XXX: this could be oob
            if input_bytes[i+1] == 0x5c {
                cur.push(0x5c);
                i += 2;
                continue;
            }

            if input_bytes[i+1] >= '0' as u8 && input_bytes[i+1] <= '2' as u8 {
                let n = read_dec(&input, i).unwrap(); // omg!
                cur.push(n);
                i += 4;
                continue;
            }
        }

        if b == '.' as u8 {
            labels.push(cur);
            cur = Vec::new();
            i += 1;
            continue;
        }

        cur.push(b);
        i += 1;
    }
    labels.push(cur);
    labels
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rfc4343_to_wire() {
        let input = "foo.example.net.";
        let expected: Vec<Vec<u8>> = vec![
            "foo".bytes().collect(),
            "example".bytes().collect(),
            "net".bytes().collect(),
            "".bytes().collect(),
        ];
        assert_eq!(rfc4343_to_wire(&input), expected);
    }

    #[test]
    fn test_rfc4343_to_wire_unusual() {
        let input = r"a\000\\\255z";
        let expected: Vec<Vec<u8>> = vec![
            vec![0x61, 0x00, 0x5c, 0xff, 0x7a],
        ];
        assert_eq!(rfc4343_to_wire(&input), expected);
    }

    #[test]
    fn test_read_dec() {
        let input = r"\046";
        let expected = 0x2e;

        assert_eq!(read_dec(&input, 0).unwrap(), expected);
    }
}
