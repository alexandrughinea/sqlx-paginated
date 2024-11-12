use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum QuerySortDirection {
    Ascending,
    #[default]
    Descending,
}
