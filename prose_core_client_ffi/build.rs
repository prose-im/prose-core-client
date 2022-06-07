fn main() {
    uniffi_build::generate_scaffolding("./src/ProseCoreClientFFI.udl").unwrap();
}
