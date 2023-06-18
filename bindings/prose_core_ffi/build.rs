fn main() {
    uniffi::generate_scaffolding("./src/prose_core_ffi.udl").expect("Failed to build UDL file");
}
