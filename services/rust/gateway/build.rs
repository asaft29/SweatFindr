fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .compile_protos(
            &["../auth-service/proto/auth.proto"],
            &["../auth-service/proto"],
        )?;

    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .compile_protos(
            &["../email-service/proto/email.proto"],
            &["../email-service/proto"],
        )?;

    Ok(())
}
