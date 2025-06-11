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
    fn test_simple_function_check() {
        // Sprawdź czy więcej funkcji się wygenerowało
        let pedersen_ptr = pedersen_hash as *const ();

        println!("pedersen_hash: {:p}", pedersen_ptr);

        assert!(!pedersen_ptr.is_null());
    }
}
