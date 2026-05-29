use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    let mut prost_config = tonic_prost_build::Config::new();
    prost_config.protoc_executable(protoc);

    tonic_prost_build::configure().compile_with_config(
        prost_config,
        &["proto/uuid_service.proto"],
        &["proto"],
    )?;
    println!("cargo:rerun-if-changed=proto/uuid_service.proto");
    Ok(())
}
