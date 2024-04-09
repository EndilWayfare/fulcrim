#[macro_export]
macro_rules! delegate_to_sql_nullable {
    ($ty: ty) => {
        impl<ST, DB> diesel::serialize::ToSql<diesel::sql_types::Nullable<ST>, DB> for $ty
        where
            DB: diesel::backend::Backend,
            $ty: diesel::serialize::ToSql<ST, DB>,
            ST: 'static,
        {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, DB>,
            ) -> diesel::serialize::Result {
                ToSql::<ST, DB>::to_sql(self, out)
            }
        }
    };
}

pub use delegate_to_sql_nullable;

#[macro_export]
macro_rules! impl_as_expression_queryable {
    (@as_expression $ty: ty, $st: ty $(, ($st2: ident) => $rt: ty)? $(, ref $lt: lifetime)?) => {
        impl<$($lt,)? $($st2)?> AsExpression<$st> for $(&$lt)? $ty $(
        where
            $st2: SingleValue,
            $rt: AsExpression<$st>
        )?
        {
            type Expression = Bound<$st, Self>;

            fn as_expression(self) -> Self::Expression {
                Bound::new(self)
            }
        }
    };

    (@as_expression $ty: ty, Nullable<$st: ty>, $rt: ty $(, ref $lt: lifetime)?) => {};

    (@as_expression $ty: ty, $st: ty $(, $rt: ty)? $(, ref $lt: lifetime)?) => {
        $crate::impl_as_expression_queryable! {@as_expression $ty, $st $(, (ST) => $rt)? $(, ref $lt)?}
    };

    (@impl $ty: ty, $st: ty $(, $st2: ty => $rt: ty)?) => {
        ::paste::paste! {
            #[allow(non_snake_case)]
            mod [<impl_diesel_for_ $ty>] {
                use super::*;

                use diesel::backend::Backend;
                use diesel::deserialize::{self, FromSql, Queryable};
                use diesel::internal::derives::as_expression::Bound;
                use diesel::expression::AsExpression;

                #[allow(unused_imports)]
                use diesel::sql_types::{Nullable, SingleValue};

                $crate::impl_as_expression_queryable! {@as_expression $ty, $st $(, $rt)?}
                $crate::impl_as_expression_queryable! {@as_expression $ty, Nullable<$st> $(, $rt)?}
                $crate::impl_as_expression_queryable! {@as_expression $ty, $st $(, $rt)?, ref 'a}
                $crate::impl_as_expression_queryable! {@as_expression $ty, Nullable<$st> $(, $rt)?, ref 'a}

                impl<$($st2,)? DB> Queryable<$st, DB> for $ty
                where
                    DB: Backend,
                    $($st2: SingleValue,)?
                    $ty: FromSql<$st, DB>,
                {
                    type Row = Self;

                    fn build(row: Self::Row) -> deserialize::Result<Self> {
                        Ok(row)
                    }
                }
            }
        }
    };

    (@impl $ty: ty, $st: ty $(, $rt: ty)?) => {
        $crate::impl_as_expression_queryable! {@impl $ty, $st $(, $st => $rt)?}
    };

    ($st: ty [expresses] $ty: ty) => {
        $crate::impl_as_expression_queryable! {@impl $ty, $st}
    };

    ($rt: ty [generalizes] $ty: ty) => {
        $crate::impl_as_expression_queryable! {@impl $ty, ST, $rt}
    };
}

pub use impl_as_expression_queryable;

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
        ::paste::paste! {
            #[allow(non_snake_case)]
            mod [<impl_diesel_for_ $ty>] {
                use super::$ty;

                use diesel::backend::Backend;
                use diesel::deserialize::{self, FromSql};
                use diesel::serialize::{self, ToSql};
                use diesel::sql_types::Int4;
                use diesel::query_builder::bind_collector::RawBytesBindCollector;

                impl<DB> FromSql<Int4, DB> for $ty
                where
                    DB: Backend,
                    i32: FromSql<Int4, DB>,
                {
                    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
                        // TODO: Support alternate wrapping strategies
                        i32::from_sql(bytes).map(|n| n as u16).map($ty::new)
                    }
                }

                impl<DB> ToSql<Int4, DB> for $ty
                where
                    DB: for<'a> Backend<BindCollector<'a> = RawBytesBindCollector<DB>>,
                    i32: ToSql<Int4, DB>,
                {
                    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, DB>) -> serialize::Result {
                        // TODO: Support alternate unwrapping strategies
                        i32::from(self.0).to_sql(&mut out.reborrow())
                    }
                }

                $crate::delegate_to_sql_nullable!($ty);

                $crate::impl_as_expression_queryable! {Int4 [expresses] $ty}
            }
        }
    };
}

pub use impl_diesel_for_u16_in_terms_of_i32;
