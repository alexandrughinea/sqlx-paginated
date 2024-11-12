use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::encode::IsNull;
use sqlx::{Database, Encode, Type};
use std::any::type_name;

#[derive(Clone, Debug)]
pub enum QueryDateTime {
    TimestampTz(DateTime<Utc>), // RFC3339
    Timestamp(NaiveDateTime),   // YYYY-MM-DD HH:MM:SS
    Date(NaiveDate),            // YYYY-MM-DD
    Time(NaiveTime),            // HH:MM:SS
}

impl QueryDateTime {
    pub fn parse_str(value: &str) -> Result<Self, String> {
        // Try parsing as RFC3339 (timestamp with timezone) first
        if let Ok(date_time) = DateTime::parse_from_rfc3339(value) {
            return Ok(QueryDateTime::TimestampTz(date_time.with_timezone(&Utc)));
        }

        // Try parsing as naive datetime (timestamp without timezone)
        if let Ok(date_time) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
            return Ok(QueryDateTime::Timestamp(date_time));
        }

        // Try parsing as date
        if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
            return Ok(QueryDateTime::Date(date));
        }

        // Try parsing as time
        if let Ok(time) = NaiveTime::parse_from_str(value, "%H:%M:%S") {
            return Ok(QueryDateTime::Time(time));
        }

        Err(format!("Unable to parse datetime string: {}", value))
    }

    pub fn to_sql_string<DB: Database>(&self) -> &'static str {
        match type_name::<DB>() {
            "sqlx_postgres::database::Postgres" => match self {
                QueryDateTime::TimestampTz(_) => "::timestamp with time zone",
                QueryDateTime::Timestamp(_) => "::timestamp without time zone",
                QueryDateTime::Date(_) => "::date",
                QueryDateTime::Time(_) => "::time",
            },
            "sqlx_mysql::database::MySql" | "sqlx_sqlite::database::Sqlite" => match self {
                // ⚠️ MYSQL doesn't fully support timezone, uses TIMESTAMP
                QueryDateTime::TimestampTz(_) => "CAST AS TIMESTAMP",
                QueryDateTime::Timestamp(_) => "CAST AS DATETIME",
                QueryDateTime::Date(_) => "CAST AS DATE",
                QueryDateTime::Time(_) => "CAST AS TIME",
            },
            _ => "",
        }
    }
}

impl Serialize for QueryDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            QueryDateTime::TimestampTz(date_time) => {
                serializer.serialize_str(&date_time.to_rfc3339())
            }
            QueryDateTime::Timestamp(date_time) => serializer.serialize_str(&date_time.to_string()),
            QueryDateTime::Date(date) => serializer.serialize_str(&date.to_string()),
            QueryDateTime::Time(time) => serializer.serialize_str(&time.to_string()),
        }
    }
}

impl<'de> Deserialize<'de> for QueryDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse_str(&value).map_err(serde::de::Error::custom)
    }
}

impl<DB> Type<DB> for QueryDateTime
where
    DB: Database,
    String: for<'a> Encode<'a, DB> + Type<DB>,
{
    fn type_info() -> <DB as Database>::TypeInfo {
        <String as Type<DB>>::type_info()
    }
}

impl<'q, DB> Encode<'q, DB> for QueryDateTime
where
    DB: Database,
    String: for<'a> Encode<'a, DB> + Type<DB>,
{
    fn encode_by_ref(
        &self,
        buffer: &mut <DB as Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let value = match self {
            QueryDateTime::TimestampTz(date_time) => date_time.to_rfc3339(),
            QueryDateTime::Timestamp(date_time) => date_time.to_string(),
            QueryDateTime::Date(date) => date.to_string(),
            QueryDateTime::Time(time) => time.to_string(),
        };

        <String as Encode<DB>>::encode(value, buffer)
    }
}
