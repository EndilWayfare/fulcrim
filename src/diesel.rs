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
                diesel::serialize::ToSql::<ST, DB>::to_sql(self, out)
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

#[derive(Debug)]
#[allow(dead_code)]
struct Poo(String);

imitate_as_expression_queryable!(Poo as String);

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
        const _: () = {
            use diesel::backend::Backend;
            use diesel::deserialize::{self, FromSql, FromSqlRow};
            use diesel::query_builder::bind_collector::RawBytesBindCollector;
            use diesel::serialize::{self, ToSql};
            use diesel::sql_types::Int4;

            impl<DB> FromSql<Int4, DB> for $ty
            where
                DB: Backend,
                i32: FromSql<Int4, DB>,
            {
                fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
                    // TODO: Support alternate wrapping strategies
                    i32::from_sql(bytes).map(|n| n as u16).map(<$ty>::new)
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
                    // TODO: Support alternate unwrapping strategies
                    i32::from(self.0).to_sql(&mut out.reborrow())
                }
            }

            $crate::derive_as_expression_queryable!($ty as Int4);
        };
    };
}

pub use impl_diesel_for_u16_in_terms_of_i32;
