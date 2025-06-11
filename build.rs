use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let barretenberg_build = "/home/uacias/dev/visoft/aztec/aztec-packages/barretenberg/cpp/build";

    println!("cargo:rustc-link-search={}/lib", barretenberg_build);
    println!("cargo:rustc-link-lib=static=barretenberg");
    println!("cargo:rustc-link-lib=stdc++");

    let bindings = bindgen::Builder::default()
        .header_contents(
            "wrapper.hpp",
            r#"
                #include <barretenberg/crypto/pedersen_commitment/c_bind.hpp>
                #include <barretenberg/crypto/pedersen_hash/c_bind.hpp>
                #include <barretenberg/crypto/poseidon2/c_bind.hpp>
                #include <barretenberg/crypto/blake2s/c_bind.hpp>
                #include <barretenberg/common/c_bind.hpp>
            "#,
        )
        .clang_arg(format!("-I{}/include", barretenberg_build))
        .clang_arg(format!("-I{}/_deps/tracy-src/public", barretenberg_build))
        .clang_arg(format!(
            "-I{}/_deps/msgpack-c/src/msgpack-c/include",
            barretenberg_build
        ))
        .clang_arg("-std=c++20")
        .clang_arg("-xc++")
        // Crypto functions
        .allowlist_function("pedersen_hash")
        .allowlist_function("poseidon_hash")
        .allowlist_function("blake2s")
        .allowlist_function("blake2s_to_field_")
        .allowlist_function("common_init_slab_allocator")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
