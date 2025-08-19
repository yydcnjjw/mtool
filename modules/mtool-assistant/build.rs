fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::compile_protos("proto/notify.proto")?;
    tonic_prost_build::compile_protos("proto/media_player.proto")?;
    Ok(())
}
