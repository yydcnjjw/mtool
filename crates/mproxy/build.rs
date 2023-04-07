fn main() {
    println!("cargo:rerun-if-changed=src/geosite.proto");
    protobuf_codegen::Codegen::new()
        .pure()
        .include("src/config/protos")
        .input("src/config/protos/geosite.proto")
        .cargo_out_dir("protos")
        .run_from_script();
}
