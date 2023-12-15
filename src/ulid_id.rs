pub extern crate macro_attr_2018;
pub extern crate newtype_derive_2018;
pub extern crate paste;
pub extern crate ulid;

cfg_if::cfg_if! {
    if #[cfg(feature = "serde1")] {
        pub extern crate serde;
        use serde::{Deserialize, Serialize};

        #[derive(Clone, Debug, PartialEq)]
        #[derive(Deserialize, Serialize)]
        #[serde(remote = "ulid::DecodeError")]
        pub enum UlidDecodeErrorDef {
            InvalidLength,
            InvalidChar,
        }

        #[macro_export]
        macro_rules! _ulid_id_conditional {
            (@use) => {
                use $crate::ulid_id::serde::{Deserialize, Serialize};
                use $crate::ulid_id::UlidDecodeErrorDef;
            };

            (@create_type $type: ident) => {
                #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
                #[derive(Deserialize, Serialize)]
                pub struct $type(ulid::Ulid);
            };

            (@create_error_type $error_type: ident) => {
                #[derive(Debug, Clone, PartialEq)]
                #[derive(Deserialize, Serialize)]
                pub struct $error_type(
                    #[serde(with = "UlidDecodeErrorDef")]
                    ulid::DecodeError
                );
            }
        }
    }
    else {
        #[macro_export]
        macro_rules! _ulid_id_conditional {
            (@use) => {};

            (@create_type $type: ident) => {
                #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
                pub struct $type($crate::ulid_id::ulid::Ulid);
            };

            (@create_error_type $error_type: ident) => {
                #[derive(Debug, Clone, PartialEq)]
                pub struct $error_type(ulid::DecodeError);
            }
        }
    }
}

#[macro_export]
macro_rules! ulid_id {
    ($type: ident) => {
        $crate::ulid_id::paste::paste!{
            mod [< $type:snake >] {
                use std::error::Error;
                use std::fmt::{self, Display};
                use std::str::FromStr;

                use $crate::ulid_id::newtype_derive_2018::{NewtypeDisplay, NewtypeFrom};
                use $crate::ulid_id::ulid::{self, Ulid};
                $crate::_ulid_id_conditional!(@use);

                $crate::_ulid_id_conditional!(@create_type $type);

                // NOTE: `macro_attr!` cannot resolve `macro_rules` derive macros inside
                //       `_ulid_id_conditional`; e.g. `Serialize` works, `Newtype*` doesn't.
                NewtypeDisplay!{ () pub struct $type(ulid::Ulid); }
                NewtypeFrom!{ () pub struct $type(ulid::Ulid); }

                #[allow(clippy::new_without_default)]
                impl $type {
                    pub fn new() -> Self {
                        $type(ulid::Ulid::new())
                    }
                }

                $crate::_ulid_id_conditional!(@create_error_type [<Parse $type Error>]);

                impl Display for [<Parse $type Error>] {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
                        write!(f, "Could not parse {} (Base32 decode: {})", stringify!($type), self.0)
                    }
                }

                impl Error for [<Parse $type Error>] {}

                impl From<ulid::DecodeError> for [<Parse $type Error>] {
                    fn from(err: ulid::DecodeError) -> Self {
                        Self(err)
                    }
                }

                impl FromStr for $type {
                    type Err = $crate::ulid_id::paste::paste!{[<Parse $type Error>]};

                    fn from_str(s: &str) -> Result<Self, Self::Err> {
                        Ok(Self(s.parse()?))
                    }
                }
            }
            pub use [< $type:snake >]::{$type, [<Parse $type Error>]};
        }
    };
}
