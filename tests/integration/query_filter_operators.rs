use serde::Serialize;
use sqlx_paginated::{QueryBuilder, QueryFilterCondition, QueryFilterOperator, QueryParamsBuilder};

#[derive(Serialize, Default, Debug)]
struct TestProduct {
    id: i64,
    name: String,
    price: f64,
    stock: i32,
    status: String,
    category: String,
    deleted_at: Option<String>,
}

#[cfg(feature = "postgres")]
mod postgres_tests {
    use super::*;
    use sqlx::Postgres;

    #[test]
    fn test_equality_operator() {
        let params = QueryParamsBuilder::<TestProduct>::new()
            .with_filter("status", Some("active"))
            .build();

        let (conditions, _args) = QueryBuilder::<TestProduct, Postgres>::new()
            .with_filters(&params)
            .build();

        assert_eq!(conditions.len(), 1);
        assert!(conditions[0].contains("\"status\" = $1"));
    }

    #[test]
    fn test_not_equal_operator() {
        let params = QueryParamsBuilder::<TestProduct>::new()
            .with_filter_operator("status", QueryFilterOperator::NotEqual, "deleted")
            .build();

        let (conditions, _args) = QueryBuilder::<TestProduct, Postgres>::new()
            .with_filters(&params)
            .build();

        assert_eq!(conditions.len(), 1);
        assert!(conditions[0].contains("\"status\" != $1"));
    }

    #[test]
    fn test_greater_than_operator() {
        let params = QueryParamsBuilder::<TestProduct>::new()
            .with_filter_operator("price", QueryFilterOperator::GreaterThan, "10.00")
            .build();

        let (conditions, _args) = QueryBuilder::<TestProduct, Postgres>::new()
            .with_filters(&params)
            .build();

        assert_eq!(conditions.len(), 1);
        assert!(conditions[0].contains("\"price\" > $1"));
    }

    #[test]
    fn test_greater_or_equal_operator() {
        let params = QueryParamsBuilder::<TestProduct>::new()
            .with_filter_operator("stock", QueryFilterOperator::GreaterOrEqual, "5")
            .build();

        let (conditions, _args) = QueryBuilder::<TestProduct, Postgres>::new()
            .with_filters(&params)
            .build();

        assert_eq!(conditions.len(), 1);
        assert!(conditions[0].contains("\"stock\" >= $1"));
    }

    #[test]
    fn test_less_than_operator() {
        let params = QueryParamsBuilder::<TestProduct>::new()
            .with_filter_operator("price", QueryFilterOperator::LessThan, "100.00")
            .build();

        let (conditions, _args) = QueryBuilder::<TestProduct, Postgres>::new()
            .with_filters(&params)
            .build();

        assert_eq!(conditions.len(), 1);
        assert!(conditions[0].contains("\"price\" < $1"));
    }

    #[test]
    fn test_less_or_equal_operator() {
        let params = QueryParamsBuilder::<TestProduct>::new()
            .with_filter_operator("stock", QueryFilterOperator::LessOrEqual, "100")
            .build();

        let (conditions, _args) = QueryBuilder::<TestProduct, Postgres>::new()
            .with_filters(&params)
            .build();

        assert_eq!(conditions.len(), 1);
        assert!(conditions[0].contains("\"stock\" <= $1"));
    }

    #[test]
    fn test_in_operator() {
        let params = QueryParamsBuilder::<TestProduct>::new()
            .with_filter_in("status", vec!["active", "pending", "approved"])
            .build();

        let (conditions, _args) = QueryBuilder::<TestProduct, Postgres>::new()
            .with_filters(&params)
            .build();

        assert_eq!(conditions.len(), 1);
        assert!(conditions[0].contains("\"status\" IN"));
        assert!(conditions[0].contains("$1"));
        assert!(conditions[0].contains("$2"));
        assert!(conditions[0].contains("$3"));
    }

    #[test]
    fn test_not_in_operator() {
        let params = QueryParamsBuilder::<TestProduct>::new()
            .with_filter_not_in("status", vec!["deleted", "banned"])
            .build();

        let (conditions, _args) = QueryBuilder::<TestProduct, Postgres>::new()
            .with_filters(&params)
            .build();

        assert_eq!(conditions.len(), 1);
        assert!(conditions[0].contains("\"status\" NOT IN"));
        assert!(conditions[0].contains("$1"));
        assert!(conditions[0].contains("$2"));
    }

    #[test]
    fn test_is_null_operator() {
        let params = QueryParamsBuilder::<TestProduct>::new()
            .with_filter_null("deleted_at", true)
            .build();

        let (conditions, _args) = QueryBuilder::<TestProduct, Postgres>::new()
            .with_filters(&params)
            .build();

        assert_eq!(conditions.len(), 1);
        assert!(conditions[0].contains("\"deleted_at\" IS NULL"));
    }

    #[test]
    fn test_is_not_null_operator() {
        let params = QueryParamsBuilder::<TestProduct>::new()
            .with_filter_null("deleted_at", false)
            .build();

        let (conditions, _args) = QueryBuilder::<TestProduct, Postgres>::new()
            .with_filters(&params)
            .build();

        assert_eq!(conditions.len(), 1);
        assert!(conditions[0].contains("\"deleted_at\" IS NOT NULL"));
    }

    #[test]
    fn test_like_operator() {
        let params = QueryParamsBuilder::<TestProduct>::new()
            .with_filter_operator("name", QueryFilterOperator::Like, "%laptop%")
            .build();

        let (conditions, _args) = QueryBuilder::<TestProduct, Postgres>::new()
            .with_filters(&params)
            .build();

        assert_eq!(conditions.len(), 1);
        assert!(conditions[0].contains("LOWER(\"name\") LIKE LOWER($1)"));
    }

    #[test]
    fn test_not_like_operator() {
        let params = QueryParamsBuilder::<TestProduct>::new()
            .with_filter_operator("name", QueryFilterOperator::NotLike, "%test%")
            .build();

        let (conditions, _args) = QueryBuilder::<TestProduct, Postgres>::new()
            .with_filters(&params)
            .build();

        assert_eq!(conditions.len(), 1);
        assert!(conditions[0].contains("LOWER(\"name\") NOT LIKE LOWER($1)"));
    }

    #[test]
    fn test_multiple_operators() {
        let params = QueryParamsBuilder::<TestProduct>::new()
            .with_filter_operator("price", QueryFilterOperator::GreaterThan, "10.00")
            .with_filter_operator("price", QueryFilterOperator::LessThan, "100.00")
            .with_filter_operator("stock", QueryFilterOperator::GreaterOrEqual, "1")
            .with_filter("status", Some("active"))
            .build();

        let (conditions, _args) = QueryBuilder::<TestProduct, Postgres>::new()
            .with_filters(&params)
            .build();

        // Note: The second price filter will overwrite the first
        // This is expected HashMap behavior
        assert!(conditions.len() >= 2);
    }

    #[test]
    fn test_filter_condition_constructors() {
        // Test QueryFilterCondition helper methods
        let eq = QueryFilterCondition::equal("test");
        assert_eq!(eq.operator, QueryFilterOperator::Equal);

        let ne = QueryFilterCondition::not_equal("test");
        assert_eq!(ne.operator, QueryFilterOperator::NotEqual);

        let gt = QueryFilterCondition::greater_than("10");
        assert_eq!(gt.operator, QueryFilterOperator::GreaterThan);

        let gte = QueryFilterCondition::greater_or_equal("10");
        assert_eq!(gte.operator, QueryFilterOperator::GreaterOrEqual);

        let lt = QueryFilterCondition::less_than("100");
        assert_eq!(lt.operator, QueryFilterOperator::LessThan);

        let lte = QueryFilterCondition::less_or_equal("100");
        assert_eq!(lte.operator, QueryFilterOperator::LessOrEqual);

        let in_list = QueryFilterCondition::in_list(vec!["a", "b", "c"]);
        assert_eq!(in_list.operator, QueryFilterOperator::In);
        assert_eq!(in_list.split_values(), vec!["a", "b", "c"]);

        let not_in = QueryFilterCondition::not_in_list(vec!["x", "y"]);
        assert_eq!(not_in.operator, QueryFilterOperator::NotIn);

        let is_null = QueryFilterCondition::is_null();
        assert_eq!(is_null.operator, QueryFilterOperator::IsNull);
        assert_eq!(is_null.value, None);

        let is_not_null = QueryFilterCondition::is_not_null();
        assert_eq!(is_not_null.operator, QueryFilterOperator::IsNotNull);
        assert_eq!(is_not_null.value, None);

        let like = QueryFilterCondition::like("%pattern%");
        assert_eq!(like.operator, QueryFilterOperator::Like);

        let not_like = QueryFilterCondition::not_like("%pattern%");
        assert_eq!(not_like.operator, QueryFilterOperator::NotLike);
    }
}

#[cfg(feature = "sqlite")]
mod sqlite_tests {
    use super::*;
    use sqlx::Sqlite;

    #[test]
    fn test_equality_operator_sqlite() {
        let params = QueryParamsBuilder::<TestProduct>::new()
            .with_filter("status", Some("active"))
            .build();

        let (conditions, _args) = QueryBuilder::<TestProduct, Sqlite>::new()
            .with_filters(&params)
            .build();

        assert_eq!(conditions.len(), 1);
        assert!(conditions[0].contains("\"status\" = ?"));
    }

    #[test]
    fn test_comparison_operators_sqlite() {
        let params = QueryParamsBuilder::<TestProduct>::new()
            .with_filter_operator("price", QueryFilterOperator::GreaterThan, "10.00")
            .with_filter_operator("stock", QueryFilterOperator::LessOrEqual, "100")
            .build();

        let (conditions, _args) = QueryBuilder::<TestProduct, Sqlite>::new()
            .with_filters(&params)
            .build();

        assert!(conditions.len() >= 1);
        // SQLite uses ? placeholders
        assert!(conditions.iter().any(|c| c.contains("?")));
    }

    #[test]
    fn test_in_operator_sqlite() {
        let params = QueryParamsBuilder::<TestProduct>::new()
            .with_filter_in("category", vec!["electronics", "books", "toys"])
            .build();

        let (conditions, _args) = QueryBuilder::<TestProduct, Sqlite>::new()
            .with_filters(&params)
            .build();

        assert_eq!(conditions.len(), 1);
        assert!(conditions[0].contains("\"category\" IN"));
        assert!(conditions[0].contains("?"));
    }
}

#[test]
fn test_filter_operator_to_sql() {
    assert_eq!(QueryFilterOperator::Equal.to_sql(), "=");
    assert_eq!(QueryFilterOperator::NotEqual.to_sql(), "!=");
    assert_eq!(QueryFilterOperator::GreaterThan.to_sql(), ">");
    assert_eq!(QueryFilterOperator::GreaterOrEqual.to_sql(), ">=");
    assert_eq!(QueryFilterOperator::LessThan.to_sql(), "<");
    assert_eq!(QueryFilterOperator::LessOrEqual.to_sql(), "<=");
    assert_eq!(QueryFilterOperator::In.to_sql(), "IN");
    assert_eq!(QueryFilterOperator::NotIn.to_sql(), "NOT IN");
    assert_eq!(QueryFilterOperator::IsNull.to_sql(), "IS NULL");
    assert_eq!(QueryFilterOperator::IsNotNull.to_sql(), "IS NOT NULL");
    assert_eq!(QueryFilterOperator::Like.to_sql(), "LIKE");
    assert_eq!(QueryFilterOperator::NotLike.to_sql(), "NOT LIKE");
}

#[test]
fn test_filter_operator_requires_value() {
    assert!(QueryFilterOperator::Equal.requires_value());
    assert!(QueryFilterOperator::GreaterThan.requires_value());
    assert!(QueryFilterOperator::In.requires_value());
    assert!(!QueryFilterOperator::IsNull.requires_value());
    assert!(!QueryFilterOperator::IsNotNull.requires_value());
}

#[test]
fn test_filter_operator_accepts_multiple() {
    assert!(QueryFilterOperator::In.accepts_multiple_values());
    assert!(QueryFilterOperator::NotIn.accepts_multiple_values());
    assert!(!QueryFilterOperator::Equal.accepts_multiple_values());
    assert!(!QueryFilterOperator::GreaterThan.accepts_multiple_values());
}

#[test]
fn test_filter_operator_from_str() {
    assert_eq!(
        QueryFilterOperator::from_str("eq"),
        QueryFilterOperator::Equal
    );
    assert_eq!(
        QueryFilterOperator::from_str("ne"),
        QueryFilterOperator::NotEqual
    );
    assert_eq!(
        QueryFilterOperator::from_str("gt"),
        QueryFilterOperator::GreaterThan
    );
    assert_eq!(
        QueryFilterOperator::from_str("gte"),
        QueryFilterOperator::GreaterOrEqual
    );
    assert_eq!(
        QueryFilterOperator::from_str("lt"),
        QueryFilterOperator::LessThan
    );
    assert_eq!(
        QueryFilterOperator::from_str("lte"),
        QueryFilterOperator::LessOrEqual
    );
    assert_eq!(QueryFilterOperator::from_str("in"), QueryFilterOperator::In);
    assert_eq!(
        QueryFilterOperator::from_str("nin"),
        QueryFilterOperator::NotIn
    );
    assert_eq!(
        QueryFilterOperator::from_str("is_null"),
        QueryFilterOperator::IsNull
    );
    assert_eq!(
        QueryFilterOperator::from_str("is_not_null"),
        QueryFilterOperator::IsNotNull
    );
    assert_eq!(
        QueryFilterOperator::from_str("like"),
        QueryFilterOperator::Like
    );
    assert_eq!(
        QueryFilterOperator::from_str("not_like"),
        QueryFilterOperator::NotLike
    );
    assert_eq!(
        QueryFilterOperator::from_str("invalid"),
        QueryFilterOperator::Equal
    ); // Defaults to Equal
}
