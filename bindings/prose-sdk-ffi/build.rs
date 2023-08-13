// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

fn main() {
    uniffi::generate_scaffolding("./src/prose_sdk_ffi.udl").expect("Failed to build UDL file");
}
