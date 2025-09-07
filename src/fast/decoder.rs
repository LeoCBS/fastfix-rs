// each index represent if a field is present in the fast message following template id
pub struct PresenceMap {
    bits: Vec<bool>,
}

//more explication about bitwise here https://play.rust-lang.org/?version=stable&mode=debug&edition=2024&gist=486362d64c12efab2584ce6550475521
impl PresenceMap {
    pub fn new(data: &[u8]) -> Option<(Self, Vec<u8>)> {
        let mut bits = Vec::new();
        let mut consumed = 0;

        for &byte in data {
            consumed += 1;
            let msb_end = (byte & 0x80) == 128; // MSB = continue?
            let seven_bits = byte & 0x7F;
            println!("seven bits = {seven_bits}");
            println!("more = {msb_end}");

            for i in (0..7).rev() {
                // seven_bits >> i "move" right bite index i
                // & 1 get bit less significative
                bits.push(((seven_bits >> i) & 1) == 1);
            }

            if msb_end {
                break; // last byte
            }
        }
        let remaining_bytes = &data[consumed..];
        Some((PresenceMap { bits }, Vec::from(remaining_bytes)))
    }

    pub fn is_present(&self, index: usize) -> bool {
        self.bits.get(index).copied().unwrap_or(false)
    }
}

fn template_id(data: &[u8]) -> Option<(u8, Vec<u8>)> {
    let mut consumed = 0;
    let mut value: u8 = 0;
    // example
    // [1,152]
    // just 7 bits
    // [1, 24]
    //
    // first byte 1
    // 0 << 7 = 0000 0000
    // 0000 0000 | 0000 0001 = 0000 0001
    //
    // second byte 24
    // 1 << 7 = 1000 0000
    // 1000 0000 | 0001 1000 = 1001 1000 (152 in decimal)
    for &byte in data {
        consumed += 1;
        let msb_end = (byte & 0x80) == 128; // MSB = continue?
        let seven_bits = byte & 0x7F;

        value = (value << 7) | seven_bits;

        if msb_end {
            break; // last byte
        }
    }
    let remaining_bytes = &data[consumed..];
    Some((value, Vec::from(remaining_bytes)))
}

fn get_app_ver_id_constant() -> String {
    "1128=9".to_string()
}

fn get_msg_type_const() -> String {
    "35=X".to_string()
}

fn decode_u32(data: &[u8]) -> Option<(u32, Vec<u8>)> {
    println!("data u32 {:?}", data);
    let mut consumed = 0;
    let mut value: u32 = 0;
    for &byte in data {
        consumed += 1;
        let msb_end = (byte & 0x80) != 0;
        let seven_bits = (byte & 0x7F) as u32;
        println!("seven bits = {seven_bits}");

        value = (value << 7) | seven_bits;
        println!("value {}", value);
        if msb_end {
            break; // last byte
        }
    }
    let remaining_bytes = &data[consumed..];
    Some((value, Vec::from(remaining_bytes)))
}

#[cfg(test)]
pub mod test_fast_decoder {

    use nom::AsBytes;

    use crate::fast::decoder::{self, PresenceMap};

    #[test]
    fn test_decode() {
        let fixture = setup();
        let (pmap, rem) = PresenceMap::new(&fixture.fastfix_msg_bytes).unwrap();
        assert!(pmap.is_present(0));
        let (template_id, rem) = decoder::template_id(rem.as_bytes()).unwrap();
        let exptected_template_id = 152;
        assert_eq!(template_id, exptected_template_id);

        let (msg_seq_num, _) = decoder::decode_u32(rem.as_bytes()).unwrap();
        let exptected_seq_num = 7503225;
        assert_eq!(msg_seq_num, exptected_seq_num);
    }

    struct Fixture {
        fastfix_msg_bytes: Vec<u8>,
    }

    fn setup() -> Fixture {
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();
        let fastfix_msg_bytes = vec![
            192, 1, 152, 3, 73, 122, 249, 35, 124, 43, 98, 103, 100, 61, 160, 128, 130, 123, 49,
            97, 152, 130, 177, 5, 105, 8, 32, 89, 232, 16, 107, 162, 128, 128, 128, 62, 10, 101,
            160, 14, 237, 9, 83, 123, 255, 128, 128, 128, 128, 128, 128, 128, 56, 50, 49, 56, 55,
            50, 52, 54, 56, 56, 51, 54, 179, 50, 183, 1, 133, 128, 128, 128, 128, 128, 128, 128,
            128, 128, 128, 128, 128, 128, 65, 13, 97, 152, 128, 2, 238, 128, 128, 129, 9, 83, 123,
            255, 62, 10, 101, 161, 128, 128, 128, 128, 128, 128, 128, 56, 50, 49, 56, 55, 52, 51,
            57, 51, 54, 51, 52, 179, 50, 183, 1, 141, 128, 128, 128, 128, 128, 128, 128, 128, 128,
            128, 128, 128, 128,
        ];
        Fixture {
            fastfix_msg_bytes: fastfix_msg_bytes,
        }
    }
}
