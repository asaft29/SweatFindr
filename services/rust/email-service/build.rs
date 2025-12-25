fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/email.proto")?;

    // Compile auth-service proto for gRPC client
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .compile_protos(&["../auth-service/proto/auth.proto"], &["../auth-service/proto"])?;

    Ok(())
}
