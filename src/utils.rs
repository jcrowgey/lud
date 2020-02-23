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
