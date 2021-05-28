#![cfg(feature = "ulid-id")]

use impls::impls;

fulcrim::ulid_id!(FooId);

mod id {
    use super::*;

    use ulid::Ulid;

    /// Assert consistency between `@create_type` across `cfg_if` blocks
    ///
    /// True, you could test by behavior. But this is clear and straightforward.
    mod impls {
        use super::*;

        #[test]
        fn copy() {
            assert!(impls!(FooId: Copy));
        }

        #[test]
        fn debug() {
            assert!(impls!(FooId: std::fmt::Debug));
        }

        #[test]
        fn eq() {
            assert!(impls!(FooId: Eq));
        }

        #[test]
        fn hash() {
            assert!(impls!(FooId: std::hash::Hash));
        }

        #[test]
        fn ord() {
            assert!(impls!(FooId: Ord));
        }
    }

    #[test]
    fn roundtrips_to_ulid() {
        let id = FooId::new();
        let ulid = Ulid::from(id);
        let roundtripped = FooId::from(ulid);

        assert_eq!(id, roundtripped);
    }

    #[test]
    fn display_mirrors_inner() {
        let id = FooId::new();
        let ulid = Ulid::from(id);

        assert_eq!(id.to_string(), ulid.to_string())
    }

    #[test]
    fn parses_from_ulid_string() {
        let ulid: Ulid = "01F6RK3ZQN7R5CPEBBRVRNMN2Q".parse().unwrap();
        let expected = FooId::from(ulid);
        let actual = "01F6RK3ZQN7R5CPEBBRVRNMN2Q".parse();

        assert_eq!(Ok(expected), actual)
    }

    #[test]
    fn rejects_parsing_invalid_ulid() {
        let expected = ParseFooIdError::from(ulid::DecodeError::InvalidLength);
        let actual = "NOT-ACTUALLY-A-ULID".parse::<FooId>();

        assert_eq!(Err(expected), actual)
    }

    #[cfg(feature = "serde1")]
    #[test]
    fn serde() {
        let id = FooId::new();
        let ulid_str: &'static String = Box::leak(Box::new(Ulid::from(id).to_string()));

        use serde_test::Token;
        serde_test::assert_tokens(
            &id,
            &[
                Token::NewtypeStruct { name: "FooId" },
                Token::String(&ulid_str),
            ],
        )
    }
}

mod parse_error {
    use super::*;

    /// Assert consistency between `@create_error_type` across `cfg_if` blocks
    ///
    /// True, you could test by behavior. But this is clear and straightforward.
    mod impls {
        use super::*;

        #[test]
        fn clone() {
            assert!(impls!(ParseFooIdError: Clone));
        }

        #[test]
        fn debug() {
            assert!(impls!(ParseFooIdError: std::fmt::Debug));
        }

        #[test]
        fn partial_eq() {
            assert!(impls!(ParseFooIdError: PartialEq));
        }

        #[test]
        fn error() {
            assert!(impls!(ParseFooIdError: std::error::Error));
        }
    }

    #[cfg(feature = "serde1")]
    #[test]
    fn serde() {
        let error = ParseFooIdError::from(ulid::DecodeError::InvalidChar);

        use serde_test::Token;
        serde_test::assert_tokens(
            &error,
            &[
                Token::NewtypeStruct { name: "ParseFooIdError" },
                Token::UnitVariant{name: "UlidDecodeErrorDef", variant: "InvalidChar"},
            ],
        )
    }
}
