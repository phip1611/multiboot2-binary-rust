/// Transforms a byte array in **big endian order** to an array of ASCII Hex characters.
/// Uses stack only, no heap allocation. `N` must be `bytes.len() * 2`!
/// The result doesn't contain `0x` prefix.
///
/// # Example
/// ```
/// use utils::convert::bytes_to_hex_ascii;
/// bytes_to_hex_ascii::<8>(&0xdeadbeef_u32.to_be_bytes());
/// ```
pub fn bytes_to_hex_ascii<const N: usize>(bytes: &[u8]) -> [char; N] {
    // we only use the first half of this
    let mut nibble_arr: [u8; N] = [0; N];
    let mut nibble_push_index = 0;
    let mut char_arr = ['\0'; N];
    let mut char_push_index = 0;

    (0..N).take(N / 2).map(|i| bytes[i])
        .map(|byte| ((0xf0 & byte) >> 4, 0x0f & byte))
        .for_each(|(h_nibble, l_nibble)| {
            nibble_arr[nibble_push_index] = h_nibble;
            nibble_arr[nibble_push_index + 1] = l_nibble;
            nibble_push_index += 2;
        });

    nibble_arr.iter()
        .map(|nibble| *nibble)
        .map(|nibble|
            if nibble <= 9 {
                nibble as u8 + '0' as u8
            } else {
                nibble as u8 - 10 + 'a' as u8
            } as char
        )
        .for_each(|char| {
            char_arr[char_push_index] = char;
            char_push_index += 1;
        });

    char_arr
}

#[cfg(test)]
mod tests {
    use crate::convert::bytes_to_hex_ascii;
    use std::iter::FromIterator;

    #[test]
    fn test_bytes_to_hex_ascii() {
        let input1 = [0xab];
        let input2 = [0xde, 0xad];
        let input3 = [0xde, 0xad, 0xbe, 0xef];
        let expected1 = ['a', 'b'];
        let expected2 = ['d', 'e', 'a', 'd'];
        let expected3 = ['d', 'e', 'a', 'd', 'b', 'e', 'e', 'f'];

        // Rust can also infer the generic const type argument in this case
        assert_eq!(bytes_to_hex_ascii(&input1), expected1);
        assert_eq!(bytes_to_hex_ascii::<2>(&input1), expected1);
        assert_eq!(bytes_to_hex_ascii(&input2), expected2);
        assert_eq!(bytes_to_hex_ascii::<4>(&input2), expected2);
        assert_eq!(bytes_to_hex_ascii(&input3), expected3);
        assert_eq!(bytes_to_hex_ascii::<8>(&input3), expected3);

        let input: u32 = 0x00000001;
        let expected = "00000001";
        let actual = bytes_to_hex_ascii::<8>(&input.to_be_bytes());
        let actual = String::from_iter(&actual);
        assert_eq!(expected, actual);
    }
}