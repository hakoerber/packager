fn main() {
    // trigger recompilation when a new migration is added
    println!("cargo:rerun-if-changed=migrations");

    // recompile when javascript changes, as it's embedded in the binary
    println!("cargo:rerun-if-changed=js");
}
