#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Zdefiniuj typy które wykluczyliśmy z bindgen
pub type out_buf = *mut u8;
pub type in_buf = *const u8;
pub type vec_in_buf = *const u8;
pub type out_buf32 = *mut u8;

// Include wygenerowanych bindingów
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bindings_load() {
        println!("Barretenberg bindings loaded successfully!");
    }

    #[test]
    fn test_function_exists() {
        let func_ptr = blake2s as *const ();
        println!("blake2s function pointer: {:p}", func_ptr);
        assert!(!func_ptr.is_null());
    }

    #[test]
    fn test_simple_function_check() {
        let pedersen_ptr = pedersen_hash as *const ();

        println!("pedersen_hash: {:p}", pedersen_ptr);

        assert!(!pedersen_ptr.is_null());
    }
}
