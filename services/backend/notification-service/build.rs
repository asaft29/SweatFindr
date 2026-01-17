fn main() -> anyhow::Result<()> {
    tonic_build::compile_protos("proto/auth.proto")?;
    Ok(())
}
