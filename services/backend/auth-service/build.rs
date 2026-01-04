fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/auth.proto")?;

    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .compile_protos(
            &["../email-service/proto/email.proto"],
            &["../email-service/proto"],
        )?;

    Ok(())
}
