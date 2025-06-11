#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Include wygenerowanych bindingów
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bindings_load() {
        println!("Barretenberg bindings loaded successfully!");
        // Tu możesz dodać testy konkretnych funkcji
    }

    #[test]
    fn test_function_exists() {
        // Sprawdź czy funkcja blake2s jest dostępna
        let func_ptr = blake2s as *const ();
        println!("blake2s function pointer: {:p}", func_ptr);
        assert!(!func_ptr.is_null());
    }

    #[test]
    fn test_pedersen_hash_proper() {
        unsafe {
            println!("Testing Pedersen hash with proper signature...");

            let input_data = [1u8, 2u8, 3u8, 4u8]; // przykładowe dane
            let hash_index: u32 = 0; // pierwszy hash
            let mut output = [0u8; 32];

            // vec_in_buf prawdopodobnie to struktura {ptr, len}
            // Spróbujmy przekazać bezpośrednio jako surowe wskaźniki

            // To może nie zadziałać, ale spróbujmy
            let inputs_buffer = input_data.as_ptr(); // może bindgen źle zinterpretował typ

            println!("Input: {:?}", input_data);
            println!("Hash index: {}", hash_index);
            println!("Output before: {:?}", &output[0..4]);

            pedersen_hash(
                inputs_buffer as *const _ as vec_in_buf, // cast na właściwy typ
                &hash_index as *const u32,
                output.as_mut_ptr() as out_buf,
            );

            println!("Output after: {:?}", &output[0..8]);

            if output != [0u8; 32] {
                println!("Pedersen SUCCESS!");
            }
        }
    }
}
