#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// 1) Load FFI bindings into a module, defining the same aliases
//    expected by the generated bindings.rs.
mod bindgen {
    // Aliases required by bindings.rs:
    pub type out_buf = *mut u8;
    pub type in_buf = *const u8;
    pub type vec_in_buf = *const u8;
    pub type out_buf32 = *mut u8;

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

/// Encodes a slice of 32-byte field elements (Fr) in JS-style:
/// [4-byte BE length][element1][element2]...
fn encode_vector_of_fr_js_style(fr_list: &[[u8; 32]]) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(4 + fr_list.len() * 32);
    // 4-byte big-endian length prefix
    buffer.extend_from_slice(&(fr_list.len() as u32).to_be_bytes());
    // Append each 32-byte field element
    for fr in fr_list {
        buffer.extend_from_slice(fr);
    }
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that encode_vector_of_fr_js_style produces
    /// a 4-byte BE prefix followed by the raw data
    #[test]
    fn test_encode_vector_of_fr_js_style() {
        let fr1 = [0xAAu8; 32];
        let fr2 = [0xBBu8; 32];
        let buf = encode_vector_of_fr_js_style(&[fr1, fr2]);

        // First 4 bytes: number of elements = 2 (big-endian)
        assert_eq!(&buf[0..4], &[0, 0, 0, 2]);

        // Next: 32 bytes of fr1, then 32 bytes of fr2
        assert_eq!(&buf[4..36], &fr1);
        assert_eq!(&buf[36..68], &fr2);
    }

    /// Test Poseidon2 hash for a single field element
    #[test]
    fn test_poseidon2_hash_single_fr() {
        ensure_init();

        let fr = [1u8; 32];
        let buf = encode_vector_of_fr_js_style(&[fr]);
        let mut out = [0u8; 32];

        unsafe {
            bindgen::poseidon2_hash(buf.as_ptr(), out.as_mut_ptr());
        }

        // The result must not be all zeros
        assert!(
            out.iter().any(|&b| b != 0),
            "Poseidon2 hash should be non-zero"
        );
    }

    /// Test Poseidon2 hash for two field elements
    #[test]
    fn test_poseidon2_hash_two_fr() {
        ensure_init();

        let fr1 = [0x11u8; 32];
        let fr2 = [0x22u8; 32];
        let buf = encode_vector_of_fr_js_style(&[fr1, fr2]);
        let mut out = [0u8; 32];

        unsafe {
            bindgen::poseidon2_hash(buf.as_ptr(), out.as_mut_ptr());
        }

        assert!(
            out.iter().any(|&b| b != 0),
            "Poseidon2 hash should be non-zero for two-element input"
        );
    }

    /// Test raw Blake2s FFI call with JS-style Fr encoding
    #[test]
    fn test_blake2s_single_fr_js_style() {
        ensure_init();

        // Single 32-byte field element
        let fr = [0x11u8; 32];
        let buf = encode_vector_of_fr_js_style(&[fr]);
        let mut out = [0u8; 32];

        unsafe {
            // Directly call Blake2s FFI
            bindgen::blake2s(buf.as_ptr(), out.as_mut_ptr());
        }

        // The hash result must not be all zeros
        assert!(out.iter().any(|&b| b != 0), "Blake2s should be non-zero");
    }

    /// Test raw Blake2s FFI call for two field elements
    #[test]
    fn test_blake2s_two_fr_js_style() {
        ensure_init();

        let fr1 = [0x22u8; 32];
        let fr2 = [0x33u8; 32];
        let buf = encode_vector_of_fr_js_style(&[fr1, fr2]);
        let mut out = [0u8; 32];

        unsafe {
            bindgen::blake2s(buf.as_ptr(), out.as_mut_ptr());
        }

        assert!(
            out.iter().any(|&b| b != 0),
            "Blake2s should be non-zero for two-element input"
        );
    }
}
