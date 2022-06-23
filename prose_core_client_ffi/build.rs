// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

fn main() {
    uniffi_build::generate_scaffolding("./src/ProseCoreClientFFI.udl").unwrap();

    println!("cargo:rustc-link-lib=static=strophe");
    println!("cargo:rustc-link-lib=dylib=xml2");
    println!("cargo:rustc-link-lib=dylib=expat");
}
