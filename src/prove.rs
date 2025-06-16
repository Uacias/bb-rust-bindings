use libc::free;
use std::ffi::c_void;
use std::ptr;

use acir::{native_types::WitnessMap, FieldElement};

use crate::{
    bindgen::{acir_prove_ultra_honk, acir_prove_ultra_keccak_honk},
    circuits::get_acir_buffer_uncompressed,
    execute::execute,
    witness::serialize_witness,
};

fn encode_raw_buffer(data: &[u8]) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(4 + data.len());
    buffer.extend_from_slice(&(data.len() as u32).to_be_bytes());
    buffer.extend_from_slice(data);
    buffer
}

const LEN_PREFIX: usize = 4;

pub fn prove_ultra_honk(
    circuit_bytecode: &str,
    initial_witness: WitnessMap<FieldElement>,
    recursive: bool,
    pub_inputs_amount: usize,
) -> Result<(Vec<u8>, Vec<u8>), String> {
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

    let len_be = unsafe { std::ptr::read_unaligned(out_ptr as *const [u8; LEN_PREFIX]) };
    let len = u32::from_be_bytes(len_be) as usize;

    let data_ptr = unsafe { out_ptr.add(LEN_PREFIX) };
    let proof = unsafe { std::slice::from_raw_parts(data_ptr, len).to_vec() };

    unsafe { free(out_ptr as *mut c_void) };

    let pub_inputs_len = 32;
    if proof.len() < pub_inputs_len {
        return Err("za mało bajtów, nie ma public inputs".into());
    }

    let public_inputs = proof[LEN_PREFIX..pub_inputs_len * pub_inputs_amount].to_vec();
    let raw_proof = proof[LEN_PREFIX + pub_inputs_len * pub_inputs_amount..].to_vec();

    Ok((public_inputs, raw_proof))
}
pub fn prove_ultra_keccak_honk(
    circuit_bytecode: &str,
    initial_witness: WitnessMap<FieldElement>,
    recursive: bool,
    pub_inputs_amount: usize,
) -> Result<(Vec<u8>, Vec<u8>), String> {
    let witness_stack = execute(circuit_bytecode, initial_witness)?;
    let serialized_solved_witness = serialize_witness(witness_stack)?;
    let acir_buffer_uncompressed = get_acir_buffer_uncompressed(circuit_bytecode)?;

    let acir_input = encode_raw_buffer(&acir_buffer_uncompressed);
    let witness_input = encode_raw_buffer(&serialized_solved_witness);

    let acir_ptr = acir_input.as_ptr();
    let witness_ptr = witness_input.as_ptr();

    let mut out_ptr: *mut u8 = ptr::null_mut();

    unsafe {
        acir_prove_ultra_keccak_honk(acir_ptr, witness_ptr, &mut out_ptr as *mut *mut u8);
    }

    let len_be = unsafe { std::ptr::read_unaligned(out_ptr as *const [u8; LEN_PREFIX]) };
    let len = u32::from_be_bytes(len_be) as usize;

    let data_ptr = unsafe { out_ptr.add(LEN_PREFIX) };
    let proof = unsafe { std::slice::from_raw_parts(data_ptr, len).to_vec() };

    unsafe { free(out_ptr as *mut c_void) };

    let pub_inputs_len = 32;
    if proof.len() < pub_inputs_len {
        return Err("za mało bajtów, nie ma public inputs".into());
    }

    let public_inputs = proof[LEN_PREFIX..pub_inputs_len * pub_inputs_amount].to_vec();
    let proof = proof[LEN_PREFIX + pub_inputs_len * pub_inputs_amount..].to_vec();

    Ok((public_inputs, proof))
}
