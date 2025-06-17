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

#[derive(Debug)]
pub struct ProofResponse {
    pub public_inputs: Vec<u8>,
    pub raw_proof: Vec<u8>,
    pub complete_data: Vec<u8>,
}

pub fn prove_ultra_honk(
    circuit_bytecode: &str,
    initial_witness: WitnessMap<FieldElement>,
    pub_inputs_amount: usize,
    isKeccak: bool,
) -> Result<ProofResponse, String> {
    let witness_stack = execute(circuit_bytecode, initial_witness)?;
    let serialized_solved_witness = serialize_witness(witness_stack)?;
    let acir_buffer_uncompressed = get_acir_buffer_uncompressed(circuit_bytecode)?;

    let acir_input = encode_raw_buffer(&acir_buffer_uncompressed);
    let witness_input = encode_raw_buffer(&serialized_solved_witness);

    let acir_ptr = acir_input.as_ptr();
    let witness_ptr = witness_input.as_ptr();

    let mut out_ptr: *mut u8 = ptr::null_mut();

    if isKeccak {
        unsafe {
            acir_prove_ultra_keccak_honk(acir_ptr, witness_ptr, &mut out_ptr as *mut *mut u8);
        }
    } else {
        unsafe {
            acir_prove_ultra_honk(acir_ptr, witness_ptr, &mut out_ptr as *mut *mut u8);
        }
    }

    // Skip both 4-byte prefixes
    let data_start = 2 * LEN_PREFIX; // 8 bytes

    // Read len
    let inner_len_be =
        unsafe { std::ptr::read_unaligned(out_ptr.add(LEN_PREFIX) as *const [u8; LEN_PREFIX]) };
    let inner_len = u32::from_be_bytes(inner_len_be) as usize;

    // Copy data from 8th byte
    let proof_with_pub_inputs =
        unsafe { std::slice::from_raw_parts(out_ptr.add(data_start), inner_len).to_vec() };

    // Copy all (with prefixes) dla complete_data
    let complete_data =
        unsafe { std::slice::from_raw_parts(out_ptr, data_start + inner_len).to_vec() };

    unsafe { free(out_ptr as *mut c_void) };

    // Public inputs: each  32 bytes
    let pub_inputs_total_len = 32 * pub_inputs_amount;
    let public_inputs = proof_with_pub_inputs[0..pub_inputs_total_len].to_vec();
    let raw_proof = proof_with_pub_inputs[pub_inputs_total_len..].to_vec();
    Ok(ProofResponse {
        public_inputs,
        raw_proof,
        complete_data,
    })
}
