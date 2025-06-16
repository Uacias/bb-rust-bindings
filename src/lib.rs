#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod barretenberg;
pub mod circuits;
pub mod execute;
pub mod prove;
pub mod witness;

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

/// encoding helpers:
fn encode_vector_of_fr(fr_list: &[[u8; 32]]) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(4 + fr_list.len() * 32);
    buffer.extend_from_slice(&(fr_list.len() as u32).to_be_bytes());
    for fr in fr_list {
        buffer.extend_from_slice(fr);
    }
    buffer
}

fn encode_raw_buffer(data: &[u8]) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(4 + data.len());
    buffer.extend_from_slice(&(data.len() as u32).to_be_bytes());
    buffer.extend_from_slice(data);
    buffer
}

/// Parse a `vec_out_buf` (length-prefixed array of 32-byte elements)
unsafe fn parse_vec_out_buf(ptr: vec_out_buf) -> Vec<[u8; 32]> {
    if ptr.is_null() || (*ptr).is_null() {
        return Vec::new();
    }
    let data_ptr = *ptr;
    let len_bytes = std::slice::from_raw_parts(data_ptr, 4);
    let len = u32::from_be_bytes([len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3]]) as usize;
    let mut result = Vec::with_capacity(len);
    for i in 0..len {
        let offset = 4 + i * 32;
        let mut element = [0u8; 32];
        element.copy_from_slice(std::slice::from_raw_parts(data_ptr.add(offset), 32));
        result.push(element);
    }
    result
}

/// Blake2s hash of field elements
pub fn blake2s_safe(input: &[[u8; 32]]) -> [u8; 32] {
    let buf = encode_vector_of_fr(input);
    let mut out = [0u8; 32];
    unsafe {
        bindgen::blake2s(buf.as_ptr(), out.as_mut_ptr());
    }
    out
}

/// Blake2s → field
pub fn blake2s_to_field_safe(input: &[[u8; 32]]) -> [u8; 32] {
    let buf = encode_vector_of_fr(input);
    let mut out = [0u8; 32];
    unsafe {
        bindgen::blake2s_to_field_(buf.as_ptr(), out.as_mut_ptr());
    }
    out
}

/// Poseidon2 hash of field elements
pub fn poseidon2_hash_safe(input: &[[u8; 32]]) -> [u8; 32] {
    let buf = encode_vector_of_fr(input);
    let mut out = [0u8; 32];
    unsafe {
        bindgen::poseidon2_hash(buf.as_ptr(), out.as_mut_ptr());
    }
    out
}

/// Poseidon2 hashes of multiple element-sets
pub fn poseidon2_hashes_safe(inputs: &[Vec<[u8; 32]>]) -> Vec<[u8; 32]> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&(inputs.len() as u32).to_be_bytes());
    for set in inputs {
        buf.extend_from_slice(&encode_vector_of_fr(set));
    }
    let mut raw = vec![0u8; 32 * inputs.len()];
    unsafe {
        bindgen::poseidon2_hashes(buf.as_ptr(), raw.as_mut_ptr());
    }
    raw.chunks_exact(32)
        .map(|c| {
            let mut a = [0u8; 32];
            a.copy_from_slice(c);
            a
        })
        .collect()
}

/// Poseidon2 permutation → Vec<[u8;32]>
pub fn poseidon2_permutation_safe(input: &[[u8; 32]]) -> Vec<[u8; 32]> {
    let buf = encode_vector_of_fr(input);
    unsafe {
        let mut raw_ptr: *mut u8 = std::ptr::null_mut();
        bindgen::poseidon2_permutation(buf.as_ptr(), &mut raw_ptr);
        parse_vec_out_buf(&mut raw_ptr)
    }
}

/// Poseidon2 accumulate hash
pub fn poseidon2_accumulate_safe(input: &[[u8; 32]]) -> [u8; 32] {
    let buf = encode_vector_of_fr(input);
    let mut out = [0u8; 32];
    unsafe {
        bindgen::poseidon2_hash_accumulate(buf.as_ptr(), out.as_mut_ptr());
    }
    out
}

/// Pedersen hash of field elements
pub fn pedersen_hash_safe(input: &[[u8; 32]], idx: u32) -> [u8; 32] {
    let buf = encode_vector_of_fr(input);
    let mut out = [0u8; 32];
    unsafe {
        bindgen::pedersen_hash(buf.as_ptr(), &idx as *const u32, out.as_mut_ptr());
    }
    out
}

/// Pedersen hashes of multiple element-sets
pub fn pedersen_hashes_safe(input: &[[u8; 32]], idx: u32) -> Vec<[u8; 32]> {
    // 1) wejście zakodowane z 4-bajtowym prefiksem dłużści:
    let buf = encode_vector_of_fr(input);
    // 2) rezerwujemy miejsce na wyjście: 32 bajty * liczba elementów
    let mut raw = vec![0u8; 32 * input.len()];

    unsafe {
        // 3) wywołujemy FFI, dając surowy ptr do naszego bufora
        bindgen::pedersen_hashes(
            buf.as_ptr(),
            &idx as *const u32,
            raw.as_mut_ptr(), // *mut u8
        );
    }

    // 4) dzielimy płaski Vec<u8> na kawałki po 32 bajty
    raw.chunks_exact(32)
        .map(|chunk| {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(chunk);
            arr
        })
        .collect()
}

/// Pedersen hash of raw bytes
pub fn pedersen_hash_buffer_safe(input: &[u8], idx: u32) -> [u8; 32] {
    let buf = encode_raw_buffer(input);
    let mut out = [0u8; 32];
    unsafe {
        bindgen::pedersen_hash_buffer(buf.as_ptr(), &idx as *const u32, out.as_mut_ptr());
    }
    out
}

/// Pedersen commitment (x,y)
pub fn pedersen_commit_safe(input: &[[u8; 32]], ctx: u32) -> [u8; 64] {
    let buf = encode_vector_of_fr(input);
    let mut out = [0u8; 64];
    unsafe {
        bindgen::pedersen_commit(buf.as_ptr(), &ctx as *const u32, out.as_mut_ptr());
    }
    out
}

/// Initialize SRS (Structured Reference String)
pub fn srs_init_safe(g1_data: &[u8], num_points: u32, g2_data: &[u8]) {
    let num_be = num_points.to_be();
    unsafe {
        bindgen::srs_init_srs(g1_data.as_ptr(), &num_be as *const u32, g2_data.as_ptr());
    }
}

/// Initialize Grumpkin SRS
pub fn srs_init_grumpkin_safe(num: u32) {
    unsafe {
        bindgen::srs_init_grumpkin_srs(std::ptr::null(), &num);
    }
}

/// Initialize slab allocator
pub fn init_slab_allocator_safe(circuit_size: u32) {
    unsafe {
        bindgen::common_init_slab_allocator(&circuit_size);
    }
}

/// ACIR: get circuit sizes
#[repr(C)]
#[derive(Debug)]
pub struct CircuitSizes {
    pub total: u32,
    pub subgroup: u32,
}

pub fn get_circuit_sizes_safe(cs: &[u8], rec: bool, honk: bool) -> CircuitSizes {
    let buf = encode_raw_buffer(cs);
    let mut sz = CircuitSizes {
        total: 0,
        subgroup: 0,
    };
    unsafe {
        bindgen::acir_get_circuit_sizes(buf.as_ptr(), &rec, &honk, &mut sz.total, &mut sz.subgroup);
    }
    CircuitSizes {
        total: u32::from_be(sz.total),
        subgroup: u32::from_be(sz.subgroup),
    }
}

/// ACIR: prove & verify UltraHonk
pub fn acir_prove_and_verify_safe(cs: &[u8], wit: &[u8]) -> bool {
    let cs_buf = encode_raw_buffer(cs);
    let wit_buf = encode_raw_buffer(wit);
    let mut out = false;
    unsafe {
        bindgen::acir_prove_and_verify_ultra_honk(cs_buf.as_ptr(), wit_buf.as_ptr(), &mut out);
    }
    out
}

/// ACIR: load verification key
pub fn acir_load_vk_safe(ptr: in_ptr, vk: &[u8]) {
    let buf = encode_raw_buffer(vk);
    unsafe {
        bindgen::acir_load_verification_key(ptr, buf.as_ptr());
    }
}

/// ACIR: init verification key
pub fn acir_init_vk_safe(ptr: in_ptr) {
    unsafe {
        bindgen::acir_init_verification_key(ptr);
    }
}

/// ACIR: get verification key → Vec<u8>
pub fn acir_get_vk_safe(ptr: in_ptr) -> Vec<u8> {
    unsafe {
        let mut out_raw: *mut u8 = std::ptr::null_mut();
        bindgen::acir_get_verification_key(ptr, &mut out_raw);
        // assume length-prefixed
        let v = parse_vec_out_buf(&mut out_raw);
        v.into_iter().flatten().collect()
    }
}

/// ACIR: get proving key → Vec<u8>
pub fn acir_get_pk_safe(ptr: in_ptr, vec: &[u8], rec: bool) -> Vec<u8> {
    let buf = encode_raw_buffer(vec);
    unsafe {
        let mut out_raw: *mut u8 = std::ptr::null_mut();
        bindgen::acir_get_proving_key(ptr, buf.as_ptr(), &rec, &mut out_raw);
        let v = parse_vec_out_buf(&mut out_raw);
        v.into_iter().flatten().collect()
    }
}

/// ACIR: verify proof
pub fn acir_verify_proof_safe(ptr: in_ptr, proof: &[u8]) -> bool {
    let buf = encode_raw_buffer(proof);
    let mut ok = false;
    unsafe {
        bindgen::acir_verify_proof(ptr, buf.as_ptr(), &mut ok);
    }
    ok
}

/// ACIR: serialize proof into fields
pub fn acir_serialize_proof_fields_safe(
    ptr: in_ptr,
    proof: &[u8],
    num_inner: u32,
) -> Vec<[u8; 32]> {
    let buf = encode_raw_buffer(proof);
    unsafe {
        let mut raw_ptr: *mut u8 = std::ptr::null_mut();
        bindgen::acir_serialize_proof_into_fields(
            ptr,
            buf.as_ptr(),
            &num_inner as *const u32,
            &mut raw_ptr,
        );
        parse_vec_out_buf(&mut raw_ptr)
    }
}

/// ACIR: serialize VK into fields + hash
pub fn acir_serialize_vk_fields_safe(ptr: in_ptr) -> (Vec<[u8; 32]>, [u8; 32]) {
    unsafe {
        let mut raw_ptr: *mut u8 = std::ptr::null_mut();
        let mut hash = [0u8; 32];

        bindgen::acir_serialize_verification_key_into_fields(ptr, &mut raw_ptr, hash.as_mut_ptr());

        let fields = parse_vec_out_buf(&mut raw_ptr);
        (fields, hash)
    }
}

/// ACIR: prove UltraHonk → Vec<u8>
pub fn acir_prove_ultra_honk_safe(cs: &[u8], wit: &[u8]) -> Vec<u8> {
    let cs_buf = encode_raw_buffer(cs);
    let wit_buf = encode_raw_buffer(wit);
    unsafe {
        let mut out_raw: *mut u8 = std::ptr::null_mut();
        bindgen::acir_prove_ultra_honk(cs_buf.as_ptr(), wit_buf.as_ptr(), &mut out_raw);
        let v = parse_vec_out_buf(&mut out_raw);
        v.into_iter().flatten().collect()
    }
}

/// ACIR: verify UltraHonk proof
pub fn acir_verify_ultra_honk_safe(proof: &[u8], vk: &[u8]) -> bool {
    let p_buf = encode_raw_buffer(proof);
    let v_buf = encode_raw_buffer(vk);
    let mut ok = false;
    unsafe {
        bindgen::acir_verify_ultra_honk(p_buf.as_ptr(), v_buf.as_ptr(), &mut ok);
    }
    ok
}

/// ACIR: write VK UltraHonk → Vec<u8>
pub fn acir_write_vk_ultra_honk_safe(vec: &[u8]) -> Vec<u8> {
    let buf = encode_raw_buffer(vec);
    unsafe {
        let mut out_raw: *mut u8 = std::ptr::null_mut();
        bindgen::acir_write_vk_ultra_honk(buf.as_ptr(), &mut out_raw);
        let v = parse_vec_out_buf(&mut out_raw);
        v.into_iter().flatten().collect()
    }
}

/// ACIR: proof as fields
pub fn acir_proof_as_fields_ultra_honk_safe(proof: &[u8]) -> Vec<[u8; 32]> {
    let buf = encode_raw_buffer(proof);
    unsafe {
        let mut raw_ptr: *mut u8 = std::ptr::null_mut();
        bindgen::acir_proof_as_fields_ultra_honk(buf.as_ptr(), &mut raw_ptr);
        parse_vec_out_buf(&mut raw_ptr)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        barretenberg::{srs::setup_srs_from_bytecode, utils::compute_subgroup_size},
        circuits::decode_circuit,
        prove::prove_ultra_honk,
        witness::from_vec_to_witness_map,
    };
    use std::io::Write;

    use super::*;
    const FR: [u8; 32] = [1; 32];
    const MSG: &[u8] = b"hello";

    #[test]
    fn t_blake2s() {
        assert_ne!(blake2s_safe(&[FR]), [0; 32]);
    }
    #[test]
    fn t_blake2f() {
        assert_ne!(blake2s_to_field_safe(&[FR]), [0; 32]);
    }
    #[test]
    fn t_poseidon() {
        assert_ne!(poseidon2_hash_safe(&[FR, FR]), [0; 32]);
    }
    #[test]
    fn t_poseidon_hashes() {
        assert!(!poseidon2_hashes_safe(&vec![vec![FR]]).is_empty());
    }
    #[test]
    fn t_poseidon_perm() {
        assert!(!poseidon2_permutation_safe(&[FR]).is_empty());
    }
    #[test]
    fn t_poseidon_acc() {
        assert_ne!(poseidon2_accumulate_safe(&[FR]), [0; 32]);
    }

    #[test]
    fn t_pedersen_hash() {
        assert_ne!(pedersen_hash_safe(&[FR], 0), [0; 32]);
    }
    #[test]
    fn t_pedersen_hashes() {
        assert!(!pedersen_hashes_safe(&[FR], 0).is_empty());
    }
    #[test]
    fn t_pedersen_buf() {
        assert_ne!(pedersen_hash_buffer_safe(MSG, 0), [0; 32]);
    }
    #[test]
    fn t_pedersen_commit() {
        assert_ne!(pedersen_commit_safe(&[FR], 0), [0; 64]);
    }

    #[test]
    fn t_slab() {
        init_slab_allocator_safe(1);
    }
    #[test]
    fn t_slab2() {
        init_slab_allocator_safe(2);
    }
    #[test]
    fn t_slab4() {
        init_slab_allocator_safe(4);
    }
    #[test]
    fn t_slab8() {
        init_slab_allocator_safe(8);
    }
    #[test]
    fn t_slab16() {
        init_slab_allocator_safe(16);
    }

    const BYTECODE: &str = "H4sIAAAAAAAA/7VSWw4CIQzkoVG/Ze/R8ljKn1eRyN7/CMYICWH7t+wkZAhthpmCFH+oun64VJZij9bzqgzHgL2Wg9X7Em1Bh2+wKVMAH/JKSBgofCw5V8hTTDlFSOhdwS0kt1UxOc8XSCbzKeHV5AGozvhMXe5TSOZsrD0GXrq6njjLpm/O0Ycbk3Hp9mbIyb0DHETT05WvYg811FrvffAn5/vD0Ytm7mp4VjbdWZvnF3hgTpCVBAAA";
    // const BYTECODE: &str = "H4sIAAAAAAAA/62QQQqAMAwErfigpEna5OZXLLb/f4KKLZbiTQdCQg7Dsm66mc9x00O717rhG9ico5cgMOfoMxJu4C2pAEsKioqisnslysoaLVkEQ6aMRYxKFc//ZYQr29L10XfhXv4jB52E+OpMAQAA";

    #[test]
    fn test_acir_get_circuit_size() {
        let (_, constraint_system_buf) = decode_circuit(BYTECODE).unwrap();
        let circuit_sizes = get_circuit_sizes_safe(&constraint_system_buf, false, false);
        assert_eq!(circuit_sizes.total, 22);
        assert_eq!(circuit_sizes.subgroup, 32);
    }

    #[test]
    fn test_compute_subgroup_size() {
        let mut subgroup_size = compute_subgroup_size(22);
        assert_eq!(subgroup_size, 32);

        subgroup_size = compute_subgroup_size(50);
        assert_eq!(subgroup_size, 64);

        subgroup_size = compute_subgroup_size(100);
        assert_eq!(subgroup_size, 128);

        subgroup_size = compute_subgroup_size(1000);
        assert_eq!(subgroup_size, 1024);

        subgroup_size = compute_subgroup_size(10000);
        assert_eq!(subgroup_size, 16384);

        subgroup_size = compute_subgroup_size(100000);
        assert_eq!(subgroup_size, 131072);

        subgroup_size = compute_subgroup_size(200000);
        assert_eq!(subgroup_size, 262144);

        subgroup_size = compute_subgroup_size(500000);
        assert_eq!(subgroup_size, 524288);

        subgroup_size = compute_subgroup_size(1000000);
        assert_eq!(subgroup_size, 1048576);
    }

    #[tokio::test]
    async fn test_prove_and_verify_ultra_honk() {
        // 1) Setup SRS (async)
        setup_srs_from_bytecode(BYTECODE, None, true).await.unwrap();

        // 2) Prepare witness
        let initial_witness = from_vec_to_witness_map(vec![5 as u128, 6 as u128]).unwrap();

        // 3) Generuj proof
        let start = std::time::Instant::now();
        let proof =
            prove_ultra_honk(BYTECODE, initial_witness, true).expect("prove_ultra_honk failed");
        println!("ultra honk proof generation time: {:?}", start.elapsed());

        let proof_fields = acir_proof_as_fields_ultra_honk_safe(&proof);
        println!("proof_fields {:?}", proof_fields);

        let hex = proof
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join("");

        let mut file =
            std::fs::File::create("ultra_honk_proof.hex").expect("failed to create proof file");
        writeln!(file, "{}", hex).expect("failed to write proof");
        println!("Proof written to ultra_honk_proof.hex");
    }
}
