use jsonptr::Pointer;
use std::{fs, path::Path};
use typify::{TypeSpace, TypeSpaceSettings};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=./service-protocol/dev/restate/service/protocol.proto");
    println!("cargo:rerun-if-changed=./proto/dev/restate/service/rust.proto");
    prost_build::Config::new()
        .bytes(["."])
        .protoc_arg("--experimental_allow_proto3_optional")
        .enum_attribute(
            "protocol.ServiceProtocolVersion",
            "#[derive(::serde::Serialize, ::serde::Deserialize, ::strum_macros::FromRepr)]",
        )
        .compile_protos(
            &[
                "service-protocol/dev/restate/service/protocol.proto",
                "service-protocol/dev/restate/service/discovery.proto",
                "proto/dev/restate/service/rust.proto",
            ],
            &["service-protocol", "proto"],
        )?;

    println!("cargo:rerun-if-changed=./service-protocol/endpoint_manifest_schema.json");
    let mut parsed_content: serde_json::Value = serde_json::from_reader(
        std::fs::File::open("./service-protocol/endpoint_manifest_schema.json").unwrap(),
    )
        .unwrap();

    Pointer::parse(
        "#/properties/services/items/properties/handlers/items/properties/input/default",
    )
        .unwrap()
        .delete(&mut parsed_content);
    Pointer::parse(
        "#/properties/services/items/properties/handlers/items/properties/input/examples",
    )
        .unwrap()
        .delete(&mut parsed_content);
    Pointer::parse(
        "#/properties/services/items/properties/handlers/items/properties/output/default",
    )
        .unwrap()
        .delete(&mut parsed_content);
    Pointer::parse(
        "#/properties/services/items/properties/handlers/items/properties/output/examples",
    )
        .unwrap()
        .delete(&mut parsed_content);

    // Instantiate type space and run code-generation
    let mut type_space =
        TypeSpace::new(TypeSpaceSettings::default().with_derive("Clone".to_owned()));
    type_space
        .add_root_schema(serde_json::from_value(parsed_content).unwrap())
        .unwrap();

    let contents = format!(
        "{}\n{}",
        "use serde::{Deserialize, Serialize};",
        prettyplease::unparse(&syn::parse2::<syn::File>(type_space.to_stream()).unwrap())
    );

    let mut out_file = Path::new("src").to_path_buf();
    out_file.push("endpoint_manifest.rs");
    fs::write(out_file, contents).unwrap();

    Ok(())
}
