fn main() {
    uniffi_build::generate_scaffolding("./ffis/ProseCoreFFI.udl").unwrap();
}
