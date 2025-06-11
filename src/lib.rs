#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Define types removed by bindgen to avoid duplicates
pub type out_buf = *mut u8;
pub type in_buf = *const u8;
pub type vec_in_buf = *const u8;
pub type out_buf32 = *mut u8;

// Include auto-generated bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Barretenberg serialization format
// Based on the C++ code, it uses msgpack format for serialization
mod serialize {
    pub fn write_length_prefixed(data: &[u8]) -> Vec<u8> {
        let mut buffer = Vec::new();

        // Method 1: Simple length prefix (4 bytes LE)
        buffer.extend_from_slice(&(data.len() as u32).to_le_bytes());
        buffer.extend_from_slice(data);

        buffer
    }

    pub fn write_msgpack_buffer(data: &[u8]) -> Vec<u8> {
        let mut buffer = Vec::new();

        // msgpack format for byte array:
        // - If length < 256: 0xc4 + 1 byte length + data
        // - If length < 65536: 0xc5 + 2 byte length (BE) + data
        // - Otherwise: 0xc6 + 4 byte length (BE) + data

        if data.len() < 256 {
            buffer.push(0xc4);
            buffer.push(data.len() as u8);
        } else if data.len() < 65536 {
            buffer.push(0xc5);
            buffer.extend_from_slice(&(data.len() as u16).to_be_bytes());
        } else {
            buffer.push(0xc6);
            buffer.extend_from_slice(&(data.len() as u32).to_be_bytes());
        }

        buffer.extend_from_slice(data);
        buffer
    }
}

// Initialize Barretenberg
pub fn init_barretenberg() {
    unsafe {
        // Initialize with a reasonable circuit size
        let circuit_size: u32 = 1 << 19; // 2^19 gates
        common_init_slab_allocator(&circuit_size as *const u32);
    }
}

#[cfg(test)]
mod tests {
    use super::serialize::*;
    use super::*;
    use std::sync::Once;

    // Ensure initialization happens only once
    static INIT: Once = Once::new();

    fn ensure_initialized() {
        INIT.call_once(|| {
            init_barretenberg();
        });
    }

    #[test]
    fn test_blake2s_all_formats() {
        ensure_initialized();

        unsafe {
            println!("\n=== Testing Blake2s with different serialization formats ===");

            let test_data = b"hello world"; // 11 bytes

            // Format 1: Raw data (no serialization)
            println!("\n1. Raw data (no serialization):");
            let mut output1 = [0u8; 32];
            blake2s(test_data.as_ptr(), output1.as_mut_ptr());
            print_result(&output1, "Raw");

            // Format 2: Length-prefixed (4 bytes LE)
            println!("\n2. Length-prefixed (4 bytes LE):");
            let buffer2 = write_length_prefixed(test_data);
            let mut output2 = [0u8; 32];
            blake2s(buffer2.as_ptr(), output2.as_mut_ptr());
            print_result(&output2, "Length-prefixed");

            // Format 3: msgpack format
            println!("\n3. msgpack format:");
            let buffer3 = write_msgpack_buffer(test_data);
            let mut output3 = [0u8; 32];
            blake2s(buffer3.as_ptr(), output3.as_mut_ptr());
            print_result(&output3, "msgpack");

            // Format 4: Fixed 32-byte input (common for field elements)
            println!("\n4. Fixed 32-byte field element:");
            let mut field_element = [0u8; 32];
            field_element[..test_data.len()].copy_from_slice(test_data);
            let mut output4 = [0u8; 32];
            blake2s(field_element.as_ptr(), output4.as_mut_ptr());
            print_result(&output4, "Field element");

            // Format 5: Double length prefix (sometimes used in nested structures)
            println!("\n5. Double length prefix:");
            let inner = write_length_prefixed(test_data);
            let outer = write_length_prefixed(&inner);
            let mut output5 = [0u8; 32];
            blake2s(outer.as_ptr(), output5.as_mut_ptr());
            print_result(&output5, "Double length");
        }
    }

    fn print_result(output: &[u8; 32], format_name: &str) {
        if output.iter().any(|&x| x != 0) {
            let hex = output[0..8]
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ");
            println!("✅ {} format: {}", format_name, hex);
        } else {
            println!("❌ {} format: all zeros", format_name);
        }
    }

    #[test]
    fn test_blake2s_known_value() {
        ensure_initialized();

        unsafe {
            println!("\n=== Testing Blake2s with known test vector ===");

            // Use a well-known test vector
            let test_input = b""; // Empty string
            let expected_blake2s_empty = [
                0x69, 0x21, 0x7a, 0x30, 0x79, 0x90, 0x80, 0x94, 0xe1, 0x11, 0x21, 0xd0, 0x42, 0x35,
                0x4a, 0x7c, 0x1f, 0x55, 0xb6, 0x48, 0x2c, 0xa1, 0xa5, 0x1e, 0x1b, 0x25, 0x0d, 0xfd,
                0x1e, 0xd0, 0xee, 0xf9,
            ];

            // Try with empty input
            let mut output = [0u8; 32];

            // Method 1: Direct empty pointer
            println!("\nTest 1: Empty input (direct)");
            blake2s(test_input.as_ptr(), output.as_mut_ptr());
            if output == expected_blake2s_empty {
                println!("✅ Matches expected Blake2s hash!");
            } else {
                println!("Output: {:?}", &output[0..8]);
            }

            // Method 2: With zero length prefix
            println!("\nTest 2: Empty input (with length prefix)");
            let buffer = write_length_prefixed(test_input);
            let mut output2 = [0u8; 32];
            blake2s(buffer.as_ptr(), output2.as_mut_ptr());
            if output2 == expected_blake2s_empty {
                println!("✅ Matches expected Blake2s hash!");
            } else {
                println!("Output: {:?}", &output2[0..8]);
            }
        }
    }
}
