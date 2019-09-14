pub fn empty_opt<T>(val: &T) -> Option<&T>
    where T: Default + PartialEq
{
    if val == &Default::default() {
        None
    } else {
        Some(val)
    }
}

/// Makes enums proto-buf serializable
///
/// Taken from https://gist.github.com/sheb-gregor/c853e5f2436796e6ac05a6e4576b529b
/// (but slightly modified)
#[macro_export]
macro_rules! proto_enum {
    (
        enum $name:ident {
            $($variant:ident = $val:expr),*,
        };
        $( $proto_type:tt )*
    ) => {
        #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Debug, Hash)]
        pub enum $name {
            $($variant = $val),+
        }

        impl protobuf::ProtobufEnum for $name {
            fn value(&self) -> i32 {
                *self as i32
            }

            fn from_i32(value: i32) -> Option<$name> {
                match value {
                    $($val => Some($name::$variant)),+,
                    _ => None
                }
            }

            fn values() -> &'static [Self] {
                static VALUES: &'static [$name] = &[
                    $($name::$variant),*
                ];
                VALUES
            }
        }

        impl $name {
            #[allow(dead_code)]
            pub fn as_str(&self) -> &str {
                match *self {
                    $($name::$variant => stringify!($variant)),+,
                }
            }

            #[allow(dead_code)]
            pub fn set_from_str(val: &str) -> Option<$name> {
                match val {
                    $( stringify!($variant) => Some($name::$variant)),+,
                    _ => None
                }
            }
        }

        impl ::std::default::Default for $name {
            fn default() -> Self {
                use protobuf::ProtobufEnum;
                $name::values()[0].clone()
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match *self {
                    $($name::$variant => f.write_str(stringify!($variant)) ),+,
                }
            }
        }

        impl From<$( $proto_type )*> for i32 {
            fn from(v: $( $proto_type )*) -> Self {
                use protobuf::ProtobufEnum;
                v.value()
            }
        }

        impl exonum::proto::ProtobufConvert for $name {
            /// Type of the protobuf clone of Self
            type ProtoStruct = $( $proto_type )*;

            /// Struct -> ProtoStruct
            fn to_pb(&self) -> Self::ProtoStruct {
                use protobuf::ProtobufEnum;

                let x = self.clone();
                match $( $proto_type )*::from_i32(x.value()){
                    Some(v) => v,
                    None =>  $( $proto_type )*::default()
                }
            }

            /// ProtoStruct -> Struct
            fn from_pb(pb: Self::ProtoStruct) -> Result<Self, failure::Error> {
                use protobuf::ProtobufEnum;

                match $name::from_i32(pb.value()) {
                    Some(v) => Ok(v),
                    None => Ok($name::default())
                }

            }
        }
    };
}

