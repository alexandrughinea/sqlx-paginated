#[cfg(feature = "postgres")]
use crate::paginated_query_as::internal::protection::COLUMN_PROTECTION_BLOCKED_POSTGRES;

#[cfg(feature = "sqlite")]
use crate::paginated_query_as::internal::protection::COLUMN_PROTECTION_BLOCKED_SQLITE;

use std::collections::HashSet;

/// Protects columns against SQL injection and system table access
#[derive(Debug, Clone)]
pub struct ColumnProtection {
    blocked_patterns: HashSet<String>,
    allowed_patterns: HashSet<String>,
    allowed_system_columns: HashSet<String>,
}

impl Default for ColumnProtection {
    fn default() -> Self {
        #[cfg(feature = "postgres")]
        {
            Self::for_postgres()
        }
        #[cfg(all(feature = "sqlite", not(feature = "postgres")))]
        {
            Self::for_sqlite()
        }
        #[cfg(not(any(feature = "postgres", feature = "sqlite")))]
        {
            Self::new()
        }
    }
}

impl ColumnProtection {
    pub fn new() -> Self {
        Self {
            blocked_patterns: HashSet::new(),
            allowed_patterns: HashSet::new(),
            allowed_system_columns: HashSet::new(),
        }
    }

    /// Creates protection with Postgres-specific blocked patterns
    #[cfg(feature = "postgres")]
    pub fn for_postgres() -> Self {
        let mut protection = Self::new();
        protection.blocked_patterns.extend(
            COLUMN_PROTECTION_BLOCKED_POSTGRES
                .iter()
                .map(|&s| s.to_string()),
        );
        protection
    }

    #[cfg(feature = "sqlite")]
    /// Creates protection with SQLite-specific blocked patterns
    pub fn for_sqlite() -> Self {
        let mut protection = Self::new();
        protection.blocked_patterns.extend(
            COLUMN_PROTECTION_BLOCKED_SQLITE
                .iter()
                .map(|&s| s.to_string()),
        );
        protection
    }

    #[allow(dead_code)]
    pub fn block_pattern(&mut self, pattern: impl Into<String>) {
        self.blocked_patterns.insert(pattern.into());
    }

    #[allow(dead_code)]
    pub fn allow_pattern(&mut self, pattern: impl Into<String>) {
        self.allowed_patterns.insert(pattern.into());
    }

    #[allow(dead_code)]
    pub fn allow_system_columns(&mut self, columns: impl IntoIterator<Item = impl Into<String>>) {
        self.allowed_system_columns
            .extend(columns.into_iter().map(|c| c.into()));
    }

    pub fn is_safe(&self, column_name: impl AsRef<str>) -> bool {
        let value = column_name.as_ref();

        // Check explicit allows first
        if self.allowed_system_columns.contains(value) {
            return true;
        }

        if self
            .allowed_patterns
            .iter()
            .any(|pattern| value.contains(pattern))
        {
            return true;
        }

        // Basic safety checks
        if value.is_empty()
            || !value
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
            || value.contains("..")
            || value.starts_with('.')
            || value.ends_with('.')
        {
            return false;
        }

        let lowercase = value.to_lowercase();
        !self
            .blocked_patterns
            .iter()
            .any(|pattern| lowercase.contains(pattern.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "postgres")]
    fn test_default_initialization() {
        let protection = ColumnProtection::default();

        // Should block system tables/columns by default (Postgres)
        assert!(!protection.is_safe("pg_table"));
        assert!(!protection.is_safe("information_schema.tables"));
        assert!(!protection.is_safe("pg_catalog.pg_class"));

        // Should block system columns by default
        assert!(!protection.is_safe("ctid"));
        assert!(!protection.is_safe("xmin"));
        assert!(!protection.is_safe("oid"));

        // Should allow regular columns
        assert!(protection.is_safe("user_id"));
        assert!(protection.is_safe("email_address"));
        assert!(protection.is_safe("first_name"));
    }

    #[test]
    #[cfg(feature = "postgres")]
    fn test_postgres_specific_protection() {
        let protection = ColumnProtection::for_postgres();

        // Should block Postgres system tables
        assert!(!protection.is_safe("pg_stat_activity"));
        assert!(!protection.is_safe("pg_catalog"));
        assert!(!protection.is_safe("information_schema.tables"));

        // Should block Postgres system columns
        assert!(!protection.is_safe("xmin"));
        assert!(!protection.is_safe("xmax"));
        assert!(!protection.is_safe("ctid"));
        assert!(!protection.is_safe("tableoid"));

        // Should allow regular columns
        assert!(protection.is_safe("user_id"));
        assert!(protection.is_safe("created_at"));
    }

    #[test]
    #[cfg(feature = "sqlite")]
    fn test_sqlite_specific_protection() {
        let protection = ColumnProtection::for_sqlite();

        // Should block SQLite system tables
        assert!(!protection.is_safe("sqlite_master"));
        assert!(!protection.is_safe("sqlite_schema"));
        assert!(!protection.is_safe("sqlite_temp_master"));
        assert!(!protection.is_safe("sqlite_sequence"));

        // Should block SQLite internal columns
        assert!(!protection.is_safe("rowid"));
        assert!(!protection.is_safe("_rowid_"));

        // Should block SQLite internal prefixes
        assert!(!protection.is_safe("sqlite_autoindex"));
        assert!(!protection.is_safe("sqlite_stat1"));

        // Should allow regular columns
        assert!(protection.is_safe("user_id"));
        assert!(protection.is_safe("created_at"));
        assert!(protection.is_safe("email"));

        // Should NOT block Postgres-specific patterns (different database)
        assert!(protection.is_safe("pg_table"));
        assert!(protection.is_safe("xmin"));
        assert!(protection.is_safe("ctid"));
    }

    #[test]
    fn test_custom_patterns() {
        let mut protection = ColumnProtection::new();

        // Add custom patterns
        protection.block_pattern("secret_");
        protection.block_pattern("internal_");
        protection.allow_pattern("public_");

        // Test blocked patterns
        assert!(!protection.is_safe("secret_key"));
        assert!(!protection.is_safe("internal_id"));

        // Test allowed patterns
        assert!(protection.is_safe("public_profile"));
        assert!(protection.is_safe("public_data"));
    }

    #[test]
    fn test_system_column_allowlist() {
        let mut protection = ColumnProtection::default();

        // Initially blocked
        assert!(!protection.is_safe("ctid"));
        assert!(!protection.is_safe("xmin"));

        // Allow specific system columns
        protection.allow_system_columns(vec!["ctid", "xmin"]);

        // Now allowed
        assert!(protection.is_safe("ctid"));
        assert!(protection.is_safe("xmin"));

        // Other system columns still blocked
        assert!(!protection.is_safe("cmax"));
        assert!(!protection.is_safe("oid"));
    }

    #[test]
    fn test_case_sensitivity() {
        let protection = ColumnProtection::default();

        // Blocked patterns should be case-insensitive
        assert!(!protection.is_safe("PG_TABLE"));
        assert!(!protection.is_safe("INFORMATION_SCHEMA.TABLES"));
        assert!(!protection.is_safe("pg_Catalog"));
        assert!(!protection.is_safe("CTID"));

        // Allowed columns should work with any case
        assert!(protection.is_safe("USER_ID"));
        assert!(protection.is_safe("Email_Address"));
    }

    #[test]
    fn test_special_characters() {
        let protection = ColumnProtection::default();

        // Should block SQL injection attempts
        assert!(!protection.is_safe("column;DROP TABLE users"));
        assert!(!protection.is_safe("column'--"));
        assert!(!protection.is_safe("column/**/"));
        assert!(!protection.is_safe("column;"));

        // Should block invalid characters
        assert!(!protection.is_safe("column$name"));
        assert!(!protection.is_safe("column@table"));
        assert!(!protection.is_safe("column#1"));

        // Should allow valid characters
        assert!(protection.is_safe("user_email_address"));
        assert!(protection.is_safe("table_123"));
        assert!(protection.is_safe("column_name_with_underscore"));
    }

    #[test]
    fn test_schema_qualified_names() {
        let mut protection = ColumnProtection::default();

        // Valid schema.table.column patterns
        assert!(protection.is_safe("public.users.id"));
        assert!(protection.is_safe("app.users.email"));

        // Invalid schema patterns
        assert!(!protection.is_safe("..column"));
        assert!(!protection.is_safe("schema..column"));
        assert!(!protection.is_safe(".column"));
        assert!(!protection.is_safe("column."));

        // Allow specific schema pattern
        protection.allow_pattern("myapp.");
        assert!(protection.is_safe("myapp.users.id"));
    }

    #[test]
    fn test_empty_and_whitespace() {
        let protection = ColumnProtection::default();

        // Empty and whitespace should be blocked
        assert!(!protection.is_safe(""));
        assert!(!protection.is_safe(" "));
        assert!(!protection.is_safe("\t"));
        assert!(!protection.is_safe("\n"));
    }

    #[test]
    fn test_pattern_precedence() {
        let mut protection = ColumnProtection::default();

        // Set up conflicting patterns
        protection.block_pattern("users_");
        protection.allow_pattern("users_table");
        protection.allow_system_columns(vec!["users_view"]);

        // Allowed patterns should take precedence over blocked patterns
        assert!(protection.is_safe("users_table"));
        assert!(protection.is_safe("users_view"));
        assert!(!protection.is_safe("users_secret"));
    }

    #[test]
    fn test_multiple_patterns() {
        let mut protection = ColumnProtection::new();

        // Add multiple patterns
        protection.block_pattern("temp_");
        protection.block_pattern("scratch_");
        protection.allow_pattern("approved_");
        protection.allow_pattern("verified_");

        // Test combinations
        assert!(!protection.is_safe("temp_table"));
        assert!(!protection.is_safe("scratch_data"));
        assert!(protection.is_safe("approved_users"));
        assert!(protection.is_safe("verified_accounts"));
    }

    #[test]
    fn test_realistic_scenarios() {
        let mut protection = ColumnProtection::default();
        protection.allow_system_columns(vec!["ctid"]);

        // Common table/column patterns
        assert!(protection.is_safe("users.id"));
        assert!(protection.is_safe("auth.user_id"));
        assert!(protection.is_safe("public.posts.title"));
        assert!(protection.is_safe("ctid")); // Explicitly allowed

        // System tables (should be blocked)
        assert!(!protection.is_safe("pg_stat_activity.pid"));
        assert!(!protection.is_safe("information_schema.tables.table_name"));

        // SQL injection attempts (should be blocked)
        assert!(!protection.is_safe("email; DELETE FROM users;"));
        assert!(!protection.is_safe("name WHERE 1=1;"));
        assert!(!protection.is_safe("id) UNION SELECT * FROM passwords;"));
    }
}
