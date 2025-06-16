use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn find_library(base_path: &str, lib_name: &str) -> Option<PathBuf> {
    let search_paths = vec![
        format!("{}/lib", base_path),
        format!("{}/_deps/{}-build", base_path, lib_name),
        format!("{}/_deps/{}/build", base_path, lib_name),
        format!("{}/../_deps/{}-build", base_path, lib_name),
    ];

    for path in search_paths {
        let path = Path::new(&path);
        if path.exists() {
            // Look for .a files
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    let file_str = file_name.to_string_lossy();
                    if file_str.contains(lib_name) && file_str.ends_with(".a") {
                        println!("cargo:warning=Found {} at {:?}", lib_name, entry.path());
                        return Some(path.to_path_buf());
                    }
                }
            }
        }
    }
    None
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let barretenberg_build = "/home/uacias/dev/visoft/barretenberg-rust-bindings/build";

    // Verify the build directory exists
    if !std::path::Path::new(barretenberg_build).exists() {
        panic!(
            "Barretenberg build directory not found: {}",
            barretenberg_build
        );
    }

    // Link libraries in the correct order (dependencies last)
    println!("cargo:rustc-link-search=native={}/lib", barretenberg_build);

    // Main libraries FIRST (they depend on libdeflate)
    println!("cargo:rustc-link-lib=static=barretenberg");
    println!("cargo:rustc-link-lib=static=env");

    // Then dependencies
    // Search for libdeflate
    if let Some(deflate_path) = find_library(barretenberg_build, "libdeflate") {
        println!("cargo:rustc-link-search=native={}", deflate_path.display());
        println!("cargo:rustc-link-lib=static=deflate");
    } else {
        // Try system libdeflate
        println!("cargo:warning=libdeflate not found in build directory, trying system library");
        println!("cargo:rustc-link-lib=deflate");
    }

    // System libraries
    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rustc-link-lib=pthread");
    println!("cargo:rustc-link-lib=m"); // math library

    // On some systems, you might also need:
    // println!("cargo:rustc-link-lib=gomp"); // OpenMP if used

    // Copy headers
    let result = Command::new("sh")
        .args(&[
            "copy-headers.sh",
            &format!("{}/include", barretenberg_build),
        ])
        .current_dir("/home/uacias/dev/visoft/aztec/aztec-packages/barretenberg/cpp")
        .output();

    match result {
        Ok(output) => {
            if !output.status.success() {
                println!(
                    "cargo:warning=copy-headers.sh failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            } else {
                println!("cargo:warning=Headers copied successfully");
            }
        }
        Err(e) => {
            println!("cargo:warning=Failed to run copy-headers.sh: {}", e);
        }
    }

    let bindings = bindgen::Builder::default()
        .header_contents(
            "wrapper.hpp",
            r#"
                #include <barretenberg/crypto/pedersen_commitment/c_bind.hpp>
                #include <barretenberg/crypto/pedersen_hash/c_bind.hpp>
                #include <barretenberg/crypto/poseidon2/c_bind.hpp>
                #include <barretenberg/crypto/blake2s/c_bind.hpp>
                #include <barretenberg/crypto/schnorr/c_bind.hpp>
                #include <barretenberg/srs/c_bind.hpp>
                #include <barretenberg/common/c_bind.hpp>
                #include <barretenberg/dsl/acir_proofs/c_bind.hpp>
            "#,
        )
        // Add include paths
        .clang_args([
            "-std=c++20",
            "-xc++",
            &format!("-I{}/include", barretenberg_build),
            &format!("-I{}/_deps/tracy-src/public", barretenberg_build),
            &format!(
                "-I{}/_deps/msgpack-c/src/msgpack-c/include",
                barretenberg_build
            ),
            "-DTRACY_ENABLE=OFF",
        ])
        // Block problematic types that we'll define manually
        .blocklist_type("out_buf")
        .blocklist_type("in_buf")
        .blocklist_type("vec_in_buf")
        .blocklist_type("out_buf32")
        // Crypto functions
        .allowlist_function("pedersen_commit")
        .allowlist_function("pedersen_hash")
        .allowlist_function("pedersen_hashes")
        .allowlist_function("pedersen_hash_buffer")
        .allowlist_function("poseidon2_hash")
        .allowlist_function("poseidon2_hashes")
        .allowlist_function("poseidon2_permutation")
        .allowlist_function("poseidon2_hash_accumulate")
        .allowlist_function("blake2s")
        .allowlist_function("blake2s_to_field_")
        // Schnorr signatures
        .allowlist_function("schnorr_construct_signature")
        .allowlist_function("schnorr_verify_signature")
        .allowlist_function("schnorr_multisig_create_multisig_public_key")
        .allowlist_function("schnorr_multisig_validate_and_combine_signer_pubkeys")
        .allowlist_function("schnorr_multisig_construct_signature_round_1")
        .allowlist_function("schnorr_multisig_construct_signature_round_2")
        .allowlist_function("schnorr_multisig_combine_signatures")
        // AES encryption
        .allowlist_function("aes_encrypt_buffer_cbc")
        .allowlist_function("aes_decrypt_buffer_cbc")
        // SRS and common
        .allowlist_function("srs_init_srs")
        .allowlist_function("srs_init_grumpkin_srs")
        .allowlist_function("test_threads")
        .allowlist_function("common_init_slab_allocator")
        // ACIR functions
        .allowlist_function("acir_get_circuit_sizes")
        .allowlist_function("acir_new_acir_composer")
        .allowlist_function("acir_delete_acir_composer")
        .allowlist_function("acir_init_proving_key")
        .allowlist_function("acir_create_proof")
        .allowlist_function("acir_load_verification_key")
        .allowlist_function("acir_init_verification_key")
        .allowlist_function("acir_get_verification_key")
        .allowlist_function("acir_get_proving_key")
        .allowlist_function("acir_verify_proof")
        .allowlist_function("acir_get_solidity_verifier")
        .allowlist_function("acir_serialize_proof_into_fields")
        .allowlist_function("acir_serialize_verification_key_into_fields")
        // Ultra Honk functions
        .allowlist_function("acir_prove_ultra_honk")
        .allowlist_function("acir_verify_ultra_honk")
        .allowlist_function("acir_write_vk_ultra_honk")
        .allowlist_function("acir_prove_and_verify_ultra_honk")
        .allowlist_function("acir_proof_as_fields_ultra_honk")
        // Use correct layout and derive Debug where possible
        .layout_tests(false)
        .derive_debug(true)
        .derive_default(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Debug - write to project root for inspection
    bindings
        .write_to_file("debug_bindings.rs")
        .expect("Couldn't write debug bindings!");

    println!("cargo:warning=Bindings generated successfully!");
}
