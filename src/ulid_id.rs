pub extern crate macro_attr_2018;
pub extern crate newtype_derive_2018;
pub extern crate paste;
pub extern crate ulid;

/// Creates a new Ulid even on "wasm32-unknown-unknown" target
///
/// As of the time of writing (2023-12-14), bare wasm standard still doesn't implement `sys::time`.
///
/// Workaround is to use `web-time` crate, which provides a replacement for `sys::time` that is
/// drop-in API compatible. Unfortunately, it is not DROP-IN drop-in compatible because it does not
/// impl `Into<std::time::SystemTime>` and so cannot be passed to functions from *foreign crates*.
///
/// But, on unsupported platforms, `std::time::SystemTime` still has `checked_add_duration` method,
/// so the conversion IS possible.
///
/// TODO: Find out why `web_time` doesn't already fucking support this.
///       It would be easy. `Into<std::time::SystemTime>::into` would automatically conditionally
///       compile to explicit `<web_time::web::SystemTime as Into<std::time::SystemTime>>::into` on
///       "wasm32-unknown-unknown" and to the blanket impl of `std::time::SystemTime` into itself
///       on all other platforms.
pub fn now() -> std::time::SystemTime {
    _now_impl()
}

// NOTE: Exact `cfg` guard taken directly from `web-time`
cfg_if::cfg_if! {
    if #[cfg(all(
        target_family = "wasm",
        not(any(target_os = "emscripten", target_os = "wasi"))
    ))] {
        fn _now_impl() -> std::time::SystemTime {
            let duration = web_time::SystemTime::UNIX_EPOCH
                .elapsed()
                .expect(
                    "The system thinks it is currently at or before Unix epoch, so you have bigger problems"
                );
            std::time::UNIX_EPOCH.checked_add(duration).unwrap()
        }
    }
    else {
        fn _now_impl() -> std::time::SystemTime {
            std::time::SystemTime::now()
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "serde")] {
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

cfg_if::cfg_if! {
    if #[cfg(feature = "diesel")] {
        pub extern crate uuid;

        #[macro_export]
        macro_rules! impl_diesel_for_ulid_newtype {
            ($ty: ty) => {
                paste::paste! {
                    #[allow(non_snake_case)]
                    mod [<impl_diesel_for_ $ty>] {
                        use super::$ty;

                        use $crate::diesel::diesel;
                        use diesel::backend::Backend;
                        use diesel::deserialize::{self, FromSql, FromSqlRow};
                        use diesel::serialize::{self, ToSql};
                        use diesel::sql_types;
                        use diesel::query_builder::bind_collector::RawBytesBindCollector;
                        use $crate::ulid_id::ulid::Ulid;
                        use $crate::ulid_id::uuid::Uuid;

                        impl<DB> FromSql<sql_types::Uuid, DB> for $ty
                        where
                            DB: Backend,
                            Uuid: FromSql<sql_types::Uuid, DB>,
                        {
                            fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
                                Uuid::from_sql(bytes).map(Ulid::from).map($ty::from)
                            }
                        }

                        impl<DB> ToSql<sql_types::Uuid, DB> for $ty
                        where
                            DB: for<'a> Backend<BindCollector<'a> = RawBytesBindCollector<DB>>,
                            Uuid: ToSql<sql_types::Uuid, DB>,
                        {
                            fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, DB>) -> serialize::Result {
                                Uuid::from(Ulid::from(*self)).to_sql(&mut out.reborrow())
                            }
                        }

                        $crate::derive_as_expression_queryable!($ty as sql_types::Uuid);
                    }
                }
            };
        }

        pub use impl_diesel_for_ulid_newtype;

        #[macro_export]
        macro_rules! _ulid_id_diesel {
            ($ty: ty) => {
                $crate::ulid_id::impl_diesel_for_ulid_newtype!($ty);
            };
        }
    } else {
        #[macro_export]
        macro_rules! _ulid_id_diesel {
            ($ty: ty) => {};
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
                $crate::_ulid_id_diesel!($type);

                // NOTE: `macro_attr!` cannot resolve `macro_rules` derive macros inside
                //       `_ulid_id_conditional`; e.g. `Serialize` works, `Newtype*` doesn't.
                NewtypeDisplay!{ () pub struct $type(ulid::Ulid); }
                NewtypeFrom!{ () pub struct $type(ulid::Ulid); }

                #[allow(clippy::new_without_default)]
                impl $type {
                    pub fn new() -> Self {
                        $type(ulid::Ulid::from_datetime($crate::ulid_id::now()))
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
