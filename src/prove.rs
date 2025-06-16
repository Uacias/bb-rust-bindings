use libc::free;
use std::ffi::c_void;
use std::ptr;

use acir::{native_types::WitnessMap, FieldElement};

use crate::{
    bindgen::acir_prove_ultra_honk, circuits::get_acir_buffer_uncompressed, execute::execute,
    witness::serialize_witness,
};

fn encode_raw_buffer(data: &[u8]) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(4 + data.len());
    buffer.extend_from_slice(&(data.len() as u32).to_be_bytes());
    buffer.extend_from_slice(data);
    buffer
}

pub fn prove_ultra_honk(
    circuit_bytecode: &str,
    initial_witness: WitnessMap<FieldElement>,
    recursive: bool,
) -> Result<Vec<u8>, String> {
    let witness_stack = execute(circuit_bytecode, initial_witness)?;
    let serialized_solved_witness = serialize_witness(witness_stack)?;
    let acir_buffer_uncompressed = get_acir_buffer_uncompressed(circuit_bytecode)?;

    let acir_input = encode_raw_buffer(&acir_buffer_uncompressed);
    let witness_input = encode_raw_buffer(&serialized_solved_witness);

    let acir_ptr = acir_input.as_ptr();
    let witness_ptr = witness_input.as_ptr();

    let mut out_ptr: *mut u8 = ptr::null_mut();

    unsafe {
        acir_prove_ultra_honk(acir_ptr, witness_ptr, &mut out_ptr as *mut *mut u8);
    }

    // Odczytaj długość z prefiksu
    let len_be = unsafe { std::ptr::read_unaligned(out_ptr as *const [u8; 4]) };
    let len = u32::from_be_bytes(len_be) as usize;

    // Skopiuj CAŁOŚĆ (łącznie z prefiksem długości)
    let total_len = 4 + len;
    let proof = unsafe { std::slice::from_raw_parts(out_ptr, total_len).to_vec() };

    // Zwolnij pamięć C++
    unsafe { free(out_ptr as *mut c_void) };

    Ok(proof) // <- Zwracamy całość, tak jak w JS
}
