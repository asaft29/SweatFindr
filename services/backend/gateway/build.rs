fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .field_attribute(".", "#[serde(default)]")
        .compile_protos(
            &["../auth-service/proto/auth.proto"],
            &["../auth-service/proto"],
        )?;

    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .field_attribute(".", "#[serde(default)]")
        .compile_protos(
            &["../email-service/proto/email.proto"],
            &["../email-service/proto"],
        )?;

    Ok(())
}
