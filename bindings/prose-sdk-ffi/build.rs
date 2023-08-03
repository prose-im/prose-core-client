fn main() {
    uniffi::generate_scaffolding("./src/prose_sdk_ffi.udl").expect("Failed to build UDL file");
}
