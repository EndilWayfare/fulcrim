pub extern crate diesel;

#[macro_export]
macro_rules! delegate_to_sql_nullable {
    ($ty: ty) => {
        impl<ST, DB>
            $crate::diesel::diesel::serialize::ToSql<
                $crate::diesel::diesel::sql_types::Nullable<ST>,
                DB,
            > for $ty
        where
            DB: $crate::diesel::diesel::backend::Backend,
            $ty: $crate::diesel::diesel::serialize::ToSql<ST, DB>,
            ST: 'static,
        {
            fn to_sql<'b>(
                &'b self,
                out: &mut $crate::diesel::diesel::serialize::Output<'b, '_, DB>,
            ) -> $crate::diesel::diesel::serialize::Result {
                $crate::diesel::diesel::serialize::ToSql::<ST, DB>::to_sql(self, out)
            }
        }
    };
}

pub use delegate_to_sql_nullable;

#[macro_export]
macro_rules! derive_as_expression_queryable {
    ($ty: ty as $st: ty) => {
        const _: () = {
            use diesel::expression::AsExpression;

            #[derive(AsExpression, FromSqlRow)]
            #[diesel(foreign_derive)]
            #[diesel(sql_type = $st)]
            struct TyProxy($ty);
        };
    };
}

#[macro_export]
macro_rules! imitate_as_expression_queryable {
    (@as_expression $ty: ty, $rt: ty $(, ref $lt: lifetime)?) => {
        impl<$($lt,)? ST> AsExpression<ST> for $(&$lt)? $ty
        where
            ST: SingleValue,
            $rt: AsExpression<ST>
        {
            type Expression = Bound<ST, Self>;

            fn as_expression(self) -> Self::Expression {
                Bound::new(self)
            }
        }
    };

    ($ty: ty as $rt: ty) => {
        const _: () = {
            use $crate::diesel::diesel;
            use diesel::backend::Backend;
            use diesel::deserialize::{self, FromSql, Queryable};
            use diesel::internal::derives::as_expression::Bound;
            use diesel::expression::AsExpression;

            #[allow(unused_imports)]
            use diesel::sql_types::{Nullable, SingleValue};

            $crate::imitate_as_expression_queryable! {@as_expression $ty, $rt}
            $crate::imitate_as_expression_queryable! {@as_expression $ty, $rt, ref 'a}

            $crate::delegate_to_sql_nullable! {$ty}

            impl<ST, DB> Queryable<ST, DB> for $ty
            where
                DB: Backend,
                ST: SingleValue,
                $ty: FromSql<ST, DB>,
            {
                type Row = Self;

                fn build(row: Self::Row) -> deserialize::Result<Self> {
                    Ok(row)
                }
            }
        };
    };
}

pub use imitate_as_expression_queryable;

// TODO: This is new stuff

/// To make this actually infallible even under external database modification, you should define
/// your SQL columns in terms of a DOMAIN type, like
/// ```sql
/// -- Infallible conversion from/to unsigned 16-bit integer
/// CREATE DOMAIN u16_as_i32 AS INTEGER CHECK (VALUE >= 0 AND VALUE <= 65535);
/// ```
#[macro_export]
macro_rules! impl_diesel_for_u16_in_terms_of_i32 {
    // TODO: Support... generics?
    ($ty: ty) => {
        $crate::impl_diesel_for_u16_in_terms_of_i32!(
            $ty,
            |x| Ok(<$ty>::try_from(x)?),
            |ty: $ty| u16::try_from(ty)
        );
    };
    ($ty: ty: newtype) => {
        $crate::impl_diesel_for_u16_in_terms_of_i32!($ty, |x| Ok(<$ty>::new(x)), |ty: $ty| {
            ::core::result::Result::<_, ::core::convert::Infallible>::Ok(ty.0)
        });
    };
    ($ty: ty, $from: expr, $into: expr) => {
        const _: () = {
            use diesel::backend::Backend;
            use diesel::deserialize::{self, FromSql, FromSqlRow};
            use diesel::query_builder::bind_collector::RawBytesBindCollector;
            use diesel::serialize::{self, ToSql};
            use diesel::sql_types::Int4;
            use $crate::diesel::diesel;

            impl<DB> FromSql<Int4, DB> for $ty
            where
                DB: Backend,
                i32: FromSql<Int4, DB>,
            {
                fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
                    i32::from_sql(bytes)
                        .and_then(|x| Ok(u16::try_from(x)?))
                        .and_then($from)
                }
            }

            impl<DB> ToSql<Int4, DB> for $ty
            where
                DB: for<'a> Backend<BindCollector<'a> = RawBytesBindCollector<DB>>,
                i32: ToSql<Int4, DB>,
            {
                fn to_sql<'b>(
                    &'b self,
                    out: &mut serialize::Output<'b, '_, DB>,
                ) -> serialize::Result {
                    // TODO: Can we avoid assuming `$ty: Copy`?
                    i32::from($into(*self)?).to_sql(&mut out.reborrow())
                }
            }

            $crate::derive_as_expression_queryable!($ty as Int4);
        };
    };
}

pub use impl_diesel_for_u16_in_terms_of_i32;

// TODO: Also copypasta'd from `ruda-quote-service`
#[macro_export]
macro_rules! impl_diesel_for_enum {
    (
        $ty: ty => $st: ty {
            $($variant: ident => $repr: expr),* $(,)?
        }
    ) => {
        paste::paste! {
            #[allow(non_snake_case)]
            mod [<impl_diesel_for_ $ty>] {
                use super::*;

                use std::error::Error;
                use std::io::Write;

                use diesel::backend::Backend;
                use diesel::deserialize::{self, FromSql, FromSqlRow};
                use diesel::expression::AsExpression;
                use diesel::pg::Pg;
                use diesel::serialize::{self, IsNull, Output, ToSql};

                impl FromSql<$st, Pg> for $ty
                {
                    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
                        match bytes.as_bytes() {
                            $($repr => Ok(Self::$variant),)*
                            bytes => {
                                Err(format!(concat!("Invalid ", stringify!($ty), " {:?}"), String::from_utf8_lossy(bytes)).into())
                            }
                        }
                    }
                }

                impl ToSql<$st, Pg> for $ty
                {
                    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
                        out.write_all(match self {
                            $(Self::$variant => $repr,)*
                        })
                        .map(|_| IsNull::No)
                        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
                    }
                }

                #[derive(AsExpression, FromSqlRow)]
                #[diesel(foreign_derive)]
                #[diesel(sql_type = $st)]
                #[allow(dead_code)]
                struct TyProxy($ty);
            }
        }
    };
}

pub use impl_diesel_for_enum;
