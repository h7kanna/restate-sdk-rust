fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=./proto/protocol.proto");
    prost_build::Config::new()
        .bytes(["."])
        .protoc_arg("--experimental_allow_proto3_optional")
        .enum_attribute(
            "protocol.ServiceProtocolVersion",
            "#[derive(::serde::Serialize, ::serde::Deserialize, ::strum_macros::FromRepr)]",
        )
        .compile_protos(&["./proto/protocol.proto"], &["."])?;
    Ok(())
}
