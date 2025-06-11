use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let barretenberg_build = "/home/uacias/dev/visoft/aztec/aztec-packages/barretenberg/cpp/build";

    // Link libraries
    println!("cargo:rustc-link-search={}/lib", barretenberg_build);
    println!("cargo:rustc-link-lib=static=barretenberg");
    println!("cargo:rustc-link-lib=stdc++");

    // Kopiowanie headerów
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
        // KLUCZOWE: Dodaj Tracy i msgpack paths
        .clang_args([
            "-std=c++20",
            "-xc++",
            &format!("-I{}/include", barretenberg_build),
            &format!("-I{}/_deps/tracy-src/public", barretenberg_build), // Tracy
            &format!(
                "-I{}/_deps/msgpack-c/src/msgpack-c/include",
                barretenberg_build
            ), // msgpack
            "-DTRACY_ENABLE=OFF",                                        // Wyłącz Tracy
        ])
        // WSZYSTKIE FUNKCJE
        .blocklist_type("out_buf")
        .blocklist_type("in_buf")
        .blocklist_type("vec_in_buf")
        .blocklist_type("out_buf32")
        .allowlist_function("pedersen_commit")
        .allowlist_function("pedersen_hash")
        .allowlist_function("pedersen_hashes")
        .allowlist_function("pedersen_hash_buffer")
        .allowlist_function("poseidon_hash")
        .allowlist_function("poseidon_hashes")
        .allowlist_function("blake2s")
        .allowlist_function("blake2s_to_field_")
        .allowlist_function("schnorr_construct_signature")
        .allowlist_function("schnorr_verify_signature")
        .allowlist_function("schnorr_multisig_create_multisig_public_key")
        .allowlist_function("schnorr_multisig_validate_and_combine_signer_pubkeys")
        .allowlist_function("schnorr_multisig_construct_signature_round_1")
        .allowlist_function("schnorr_multisig_construct_signature_round_2")
        .allowlist_function("schnorr_multisig_combine_signatures")
        .allowlist_function("aes_encrypt_buffer_cbc")
        .allowlist_function("aes_decrypt_buffer_cbc")
        .allowlist_function("srs_init_srs")
        .allowlist_function("srs_init_grumpkin_srs")
        .allowlist_function("test_threads")
        .allowlist_function("common_init_slab_allocator")
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
        .allowlist_function("acir_prove_ultra_honk")
        .allowlist_function("acir_verify_ultra_honk")
        .allowlist_function("acir_write_vk_ultra_honk")
        .allowlist_function("acir_prove_and_verify_ultra_honk")
        .allowlist_function("acir_proof_as_fields_ultra_honk")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Debug - zapisz też do głównego katalogu
    bindings
        .write_to_file("debug_bindings.rs")
        .expect("Couldn't write debug bindings!");
}
