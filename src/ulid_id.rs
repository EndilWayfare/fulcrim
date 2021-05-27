#[macro_export]
macro_rules! ulid_id {
    ($type: ident) => {
        custom_derive! {
            #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
            #[derive(NewtypeDisplay, NewtypeFrom)]
            #[derive(serde::Deserialize, serde::Serialize)]
            pub struct $type(ulid::Ulid);
        }

        #[allow(clippy::new_without_default)]
        impl $type {
            pub fn new() -> Self {
                $type(ulid::Ulid::new())
            }
        }

        paste::paste!{
            #[allow(non_snake_case)]
            mod [<_parse_ $type _error>] {
                use serde::{Deserialize, Serialize};
                use $crate::model::atoms::UlidDecodeErrorDef;

                #[derive(Debug, Clone, PartialEq)]
                #[derive(Deserialize, Serialize)]
                pub struct [<Parse $type Error>](
                    #[serde(with = "UlidDecodeErrorDef")]
                    ulid::DecodeError
                );

                impl std::fmt::Display for [<Parse $type Error>] {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                        write!(f, "Could not parse {} (Base32 decode: {})", stringify!($type), self.0)
                    }
                }

                impl std::error::Error for [<Parse $type Error>] {}

                impl From<ulid::DecodeError> for [<Parse $type Error>] {
                    fn from(err: ulid::DecodeError) -> Self {
                        Self(err)
                    }
                }
            }
            pub use [<_parse_ $type _error>]::[<Parse $type Error>];
        }

        impl std::str::FromStr for $type {
            type Err = paste::paste!{[<Parse $type Error>]};

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(s.parse()?))
            }
        }
    };
}
