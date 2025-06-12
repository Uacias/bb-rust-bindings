use crate::{bindgen::acir_get_circuit_sizes, circuits::decode_circuit};

pub fn get_honk_verification_key(
    circuit_bytecode: &str,
    recursive: bool,
) -> Result<Vec<u8>, String> {
    let (_, acir_buffer_uncompressed) =
        decode_circuit(circuit_bytecode).map_err(|e| format!("Failed to decode circuit: {}", e))?;

    // acir_get_verification_key expects (in_ptr, out: *mut *mut u8)
    // This seems to need an ACIR composer pointer, not just the buffer
    // For now, let's comment this out as it needs a different approach

    // You would need to:
    // 1. Create an ACIR composer with acir_new_acir_composer
    // 2. Initialize it with the circuit
    // 3. Then get the verification key

    Err("get_honk_verification_key needs to be implemented with proper ACIR composer".to_string())
}

pub fn compute_subgroup_size(circuit_size: u32) -> u32 {
    let log_value = (circuit_size as f64).log2().ceil() as u32;
    let subgroup_size = 2u32.pow(log_value);
    subgroup_size
}

pub fn get_circuit_size(circuit_bytecode: &str, recursive: bool) -> u32 {
    let (_, acir_buffer_uncompressed) = match decode_circuit(circuit_bytecode) {
        Ok(result) => result,
        Err(_) => return 0,
    };

    let mut total: u32 = 0;
    let mut subgroup: u32 = 0;
    let honk_recursion = false;

    unsafe {
        acir_get_circuit_sizes(
            acir_buffer_uncompressed.as_ptr(),
            &recursive as *const bool,
            &honk_recursion as *const bool,
            &mut total as *mut u32,
            &mut subgroup as *mut u32,
        );
    }

    total
}

pub fn get_subgroup_size(circuit_bytecode: &str, recursive: bool) -> u32 {
    let circuit_size = get_circuit_size(circuit_bytecode, recursive);
    compute_subgroup_size(circuit_size)
}
