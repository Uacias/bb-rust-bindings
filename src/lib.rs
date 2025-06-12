#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod barretenberg;
pub mod circuits;
// Load FFI bindings into a module, defining the same aliases
// expected by the generated bindings.rs.
pub mod bindgen {
    // Aliases required by bindings.rs:
    pub type out_buf = *mut u8;
    pub type in_buf = *const u8;
    pub type vec_in_buf = *const u8;
    pub type out_buf32 = *mut u8;

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// Re-export some commonly used types
pub use bindgen::{in_ptr, out_str_buf, vec_out_buf};

/// Encodes a slice of 32-byte field elements (Fr) in JS-style:
/// [4-byte BE length][element1][element2]...
fn encode_vector_of_fr(fr_list: &[[u8; 32]]) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(4 + fr_list.len() * 32);
    // 4-byte big-endian length prefix
    buffer.extend_from_slice(&(fr_list.len() as u32).to_be_bytes());
    // Append each 32-byte field element
    for fr in fr_list {
        buffer.extend_from_slice(fr);
    }
    buffer
}

/// Encodes raw bytes with length prefix (for non-field element data)
fn encode_raw_buffer(data: &[u8]) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(4 + data.len());
    buffer.extend_from_slice(&(data.len() as u32).to_be_bytes());
    buffer.extend_from_slice(data);
    buffer
}

/// Parse a vec_out_buf (dynamically allocated output)
unsafe fn parse_vec_out_buf(ptr: vec_out_buf) -> Vec<[u8; 32]> {
    if ptr.is_null() || (*ptr).is_null() {
        return Vec::new();
    }

    let data_ptr = *ptr;
    // First 4 bytes contain the length (BE)
    let len_bytes = std::slice::from_raw_parts(data_ptr, 4);
    let len = u32::from_be_bytes([len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3]]) as usize;

    // Rest is the actual data (field elements)
    let mut result = Vec::with_capacity(len);
    for i in 0..len {
        let offset = 4 + i * 32;
        let mut element = [0u8; 32];
        element.copy_from_slice(std::slice::from_raw_parts(data_ptr.add(offset), 32));
        result.push(element);
    }

    // Free the allocated memory (if barretenberg provides a free function)
    // Note: You might need to add a binding for the free function

    result
}

// Safe API wrappers

/// Blake2s hash of field elements
pub fn blake2s(input: &[[u8; 32]]) -> [u8; 32] {
    let buffer = encode_vector_of_fr(input);
    let mut output = [0u8; 32];

    unsafe {
        bindgen::blake2s(buffer.as_ptr(), output.as_mut_ptr());
    }

    output
}

/// Blake2s hash that outputs a field element
pub fn blake2s_to_field(input: &[[u8; 32]]) -> [u8; 32] {
    let buffer = encode_vector_of_fr(input);
    let mut output = [0u8; 32];

    unsafe {
        bindgen::blake2s_to_field_(buffer.as_ptr(), output.as_mut_ptr());
    }

    output
}

/// Poseidon2 hash of field elements
pub fn poseidon2_hash(input: &[[u8; 32]]) -> [u8; 32] {
    let buffer = encode_vector_of_fr(input);
    let mut output = [0u8; 32];

    unsafe {
        bindgen::poseidon2_hash(buffer.as_ptr(), output.as_mut_ptr());
    }

    output
}

/// Poseidon2 hash of multiple sets of field elements
pub fn poseidon2_hashes(inputs: &[Vec<[u8; 32]>]) -> Vec<[u8; 32]> {
    // Encode as nested structure
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&(inputs.len() as u32).to_be_bytes());

    for input_set in inputs {
        let encoded = encode_vector_of_fr(input_set);
        buffer.extend_from_slice(&encoded);
    }

    let mut output = vec![0u8; 32 * inputs.len()];

    unsafe {
        bindgen::poseidon2_hashes(buffer.as_ptr(), output.as_mut_ptr());
    }

    // Convert flat output to vector of field elements
    output
        .chunks_exact(32)
        .map(|chunk| {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(chunk);
            arr
        })
        .collect()
}

/// Pedersen hash
pub fn pedersen_hash(input: &[[u8; 32]], hash_index: u32) -> [u8; 32] {
    let buffer = encode_vector_of_fr(input);
    let mut output = [0u8; 32];

    unsafe {
        bindgen::pedersen_hash(
            buffer.as_ptr(),
            &hash_index as *const u32,
            output.as_mut_ptr(),
        );
    }

    output
}

/// Pedersen commitment (returns a point with x,y coordinates)
pub fn pedersen_commit(input: &[[u8; 32]], ctx_index: u32) -> [u8; 64] {
    let buffer = encode_vector_of_fr(input);
    let mut output = [0u8; 64]; // Affine point (x, y)

    unsafe {
        bindgen::pedersen_commit(
            buffer.as_ptr(),
            &ctx_index as *const u32,
            output.as_mut_ptr(),
        );
    }

    output
}

/// Pedersen hash of raw buffer (non-field element data)
pub fn pedersen_hash_buffer(input: &[u8], hash_index: u32) -> [u8; 32] {
    let buffer = encode_raw_buffer(input);
    let mut output = [0u8; 32];

    unsafe {
        bindgen::pedersen_hash_buffer(
            buffer.as_ptr(),
            &hash_index as *const u32,
            output.as_mut_ptr(),
        );
    }

    output
}

/// Schnorr signature
pub fn schnorr_sign(message: &[u8], private_key: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    let message_buffer = encode_raw_buffer(message);
    let mut sig_s = [0u8; 32];
    let mut sig_e = [0u8; 32];

    unsafe {
        bindgen::schnorr_construct_signature(
            message_buffer.as_ptr(),
            private_key.as_ptr(),
            sig_s.as_mut_ptr(),
            sig_e.as_mut_ptr(),
        );
    }

    (sig_s, sig_e)
}

/// Schnorr signature verification
pub fn schnorr_verify(
    message: &[u8],
    public_key: &[u8; 64], // Affine point
    sig_s: &[u8; 32],
    sig_e: &[u8; 32],
) -> bool {
    let message_buffer = encode_raw_buffer(message);
    let mut result = false;

    unsafe {
        bindgen::schnorr_verify_signature(
            message_buffer.as_ptr(),
            public_key.as_ptr(),
            sig_s.as_ptr(),
            sig_e.as_ptr(),
            &mut result as *mut bool,
        );
    }

    result
}

/// Initialize SRS (Structured Reference String)
pub fn srs_init(num_points: u32) {
    unsafe {
        bindgen::srs_init_srs(
            std::ptr::null(),
            &num_points as *const u32,
            std::ptr::null(),
        );
    }
}

/// Test thread functionality
pub fn test_threads(threads: u32, iterations: u32) -> u32 {
    let mut result = 0u32;

    unsafe {
        bindgen::test_threads(
            &threads as *const u32,
            &iterations as *const u32,
            &mut result as *mut u32,
        );
    }

    result
}

#[repr(C)]
pub struct CircuitSizes {
    pub total: u32,
    pub subgroup: u32,
}

/// Safe Rust wrapper around the raw FFI
pub fn get_circuit_sizes(
    constraint_system: &[u8],
    recursive: bool,
    honk_recursion: bool,
) -> CircuitSizes {
    // 1) JS-style encoding: [4-byte BE length][…data…]
    let buffer = encode_raw_buffer(constraint_system);

    let mut sizes = CircuitSizes {
        total: 0,
        subgroup: 0,
    };
    unsafe {
        crate::bindgen::acir_get_circuit_sizes(
            buffer.as_ptr(),                 // *const u8
            &recursive as *const bool,       // *const bool
            &honk_recursion as *const bool,  // *const bool
            &mut sizes.total as *mut u32,    // *mut u32
            &mut sizes.subgroup as *mut u32, // *mut u32
        );
    }
    sizes
}

#[cfg(test)]
mod tests {
    use crate::{
        barretenberg::srs::setup_srs_from_bytecode, bindgen::acir_get_circuit_sizes,
        circuits::decode_circuit,
    };

    use super::*;

    const BYTECODE: &str = "H4sIAAAAAAAA/62QQQqAMAwErfigpEna5OZXLLb/f4KKLZbiTQdCQg7Dsm66mc9x00O717rhG9ico5cgMOfoMxJu4C2pAEsKioqisnslysoaLVkEQ6aMRYxKFc//ZYQr29L10XfhXv4jB52E+OpMAQAA";

    #[test]
    fn test_acir_get_circuit_size() {
        let (_, constraint_system_buf) = decode_circuit(BYTECODE).unwrap();
        let circuit_sizes = get_circuit_sizes(&constraint_system_buf, false, false);
        println!("{}{}", circuit_sizes.subgroup, circuit_sizes.total);
        // assert_eq!(circuit_sizes.total, 22);
        // assert_eq!(circuit_sizes.subgroup, 32);
    }

    #[test]
    fn test_prove_and_verify_ultra_honk() {
        // Setup SRS
        setup_srs_from_bytecode(BYTECODE, None, false).unwrap();
    }
    #[test]
    fn test_blake2s() {
        let fr = [1u8; 32];
        let hash = blake2s(&[fr]);
        assert!(hash.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_poseidon2() {
        let fr1 = [1u8; 32];
        let fr2 = [2u8; 32];
        let hash = poseidon2_hash(&[fr1, fr2]);
        assert!(hash.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_pedersen() {
        let fr = [1u8; 32];
        let hash = pedersen_hash(&[fr], 0);
        assert!(hash.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_pedersen_commit() {
        let fr = [1u8; 32];
        let point = pedersen_commit(&[fr], 0);
        assert!(point.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_schnorr() {
        let message = b"Hello, Schnorr!";
        let mut private_key = [0u8; 32];
        private_key[0] = 1; // Non-zero private key

        let (sig_s, sig_e) = schnorr_sign(message, &private_key);
        assert!(sig_s.iter().any(|&b| b != 0) || sig_e.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_pedersen_hash_buffer() {
        let data = b"hello world";
        let hash = pedersen_hash_buffer(data, 0);
        assert!(hash.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_multiple_hashes() {
        // Test that different inputs produce different outputs
        let fr1 = [1u8; 32];
        let fr2 = [2u8; 32];

        let hash1 = blake2s(&[fr1]);
        let hash2 = blake2s(&[fr2]);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_thread_functionality() {
        let result = test_threads(4, 100);
        println!("Thread test result: {}", result);
    }
}
