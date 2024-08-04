use std::io::{Read, Result};

struct RotDecoder<R: Read> {
    input: R,
    rot: u8,
}

// Implement the `Read` trait for `RotDecoder`.
impl<R: Read> Read for RotDecoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let size = self.input.read(buf).unwrap(); // calls read(buf) for type(self.input) internally

        for byte in &mut buf[..size] { // iterate over u8 array (buf)
            if byte.is_ascii_alphabetic() { 
                if byte.is_ascii_uppercase() {
                    let base = b'A';
                    *byte = base + (*byte - base + self.rot) % 26;
                } else {
                    let base = b'a';
                    *byte = base + (*byte - base + self.rot) % 26;
                }
            } 
        }
        Ok(size)
    }

    }



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joke() {
        let mut rot = RotDecoder {
            input: "Gb trg gb gur bgure fvqr!".as_bytes(), // String.as_bytes() method: ex) "hello" -> [104, 101, 108, 108, 111]
            rot: 13,
        };
        let mut ret = String::new();

        rot.read_to_string(&mut ret).unwrap();

        assert_eq!(&ret, "To get to the other side!");
    }

    #[test]
    fn binary() {
        let input: Vec<u8> = (0..=255u8).collect();
        let mut rot = RotDecoder::<&[u8]> {
            input: input.as_ref(),
            rot: 13,
        };
        let mut buf = [0u8; 256];

        assert_eq!(rot.read(&mut buf).unwrap(), 256);

        for i in 0..=255 {
            if input[i] != buf[i] {
                assert!(input[i].is_ascii_alphabetic());
                assert!(buf[i].is_ascii_alphabetic());
            }
        }
    }
}
