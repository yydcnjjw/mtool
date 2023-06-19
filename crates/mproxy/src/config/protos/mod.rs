use std::fmt::Formatter;
use std::marker::PhantomData;

use protobuf::EnumFull;
use protobuf::EnumOrUnknown;
use serde::Deserializer;
use serde::Serializer;

include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));

pub fn serialize_enum_or_unknown<E: EnumFull, S: Serializer>(
    e: &EnumOrUnknown<E>,
    s: S,
) -> Result<S::Ok, S::Error> {
    match e.enum_value() {
        Ok(v) => s.serialize_str(v.descriptor().name()),
        Err(v) => s.serialize_i32(v),
    }
}

pub fn deserialize_enum_or_unknown<'de, E: EnumFull, D: Deserializer<'de>>(
    d: D,
) -> Result<EnumOrUnknown<E>, D::Error> {
    struct DeserializeEnumVisitor<E: EnumFull>(PhantomData<E>);

    impl<'de, E: EnumFull> serde::de::Visitor<'de> for DeserializeEnumVisitor<E> {
        type Value = EnumOrUnknown<E>;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            write!(formatter, "a string, an integer or none")
        }

        fn visit_str<R>(self, v: &str) -> Result<Self::Value, R>
        where
            R: serde::de::Error,
        {
            match E::enum_descriptor().value_by_name(v) {
                Some(v) => Ok(EnumOrUnknown::from_i32(v.value())),
                None => Err(serde::de::Error::custom(format!(
                    "unknown enum value: {}",
                    v
                ))),
            }
        }

        fn visit_i32<R>(self, v: i32) -> Result<Self::Value, R>
        where
            R: serde::de::Error,
        {
            Ok(EnumOrUnknown::from_i32(v))
        }
    }

    d.deserialize_any(DeserializeEnumVisitor(PhantomData))
}
