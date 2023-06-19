use protobuf::descriptor::field_descriptor_proto::Type;
use protobuf::reflect::FieldDescriptor;
use protobuf::reflect::MessageDescriptor;
use protobuf_codegen::Customize;
use protobuf_codegen::CustomizeCallback;

fn main() {
    println!("cargo:rerun-if-changed=src/geosite.proto");

    struct GenSerde;

    impl CustomizeCallback for GenSerde {
        fn message(&self, _message: &MessageDescriptor) -> Customize {
            Customize::default().before("#[derive(::serde::Serialize, ::serde::Deserialize)]")
        }

        fn field(&self, field: &FieldDescriptor) -> Customize {
            if field.proto().type_() == Type::TYPE_ENUM {
                // `EnumOrUnknown` is not a part of rust-protobuf, so external serializer is needed.
                Customize::default().before(
                    "#[serde(serialize_with = \"crate::config::protos::serialize_enum_or_unknown\", deserialize_with = \"crate::config::protos::deserialize_enum_or_unknown\")]")
            } else {
                Customize::default()
            }
        }

        fn oneof(&self, _oneof: &protobuf::reflect::OneofDescriptor) -> Customize {
            Customize::default().before("#[derive(::serde::Serialize, ::serde::Deserialize)]")
        }

        fn special_field(&self, _message: &MessageDescriptor, _field: &str) -> Customize {
            Customize::default().before("#[serde(skip)]")
        }
    }

    protobuf_codegen::Codegen::new()
        .pure()
        .include("src/config/protos")
        .input("src/config/protos/geosite.proto")
        .cargo_out_dir("protos")
        .customize_callback(GenSerde)
        .run_from_script();
}
