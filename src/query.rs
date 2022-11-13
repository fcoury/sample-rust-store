use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct Query {
    filter: Option<Vec<QueryFilterItem>>,
    sort: Option<Vec<QuerySortItem>>,
    limit: Option<QueryLimit>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum QueryFilterItem {
    Filter(QueryFilterFilter),
    Condition(QueryFilterCondition),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryFilterFilter {
    operation: QueryFilterOperation,
    filter: QueryFilter,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryFilterCondition {
    operation: QueryFilterOperation,
    filter: Vec<QueryFilterItem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryFilter {
    field: String,
    operator: QueryFilterOperator,
    value: Value,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum QueryFilterOperation {
    And,
    Or,
    Not,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum QueryFilterOperator {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanOrEquals,
    LessThan,
    LessThanOrEquals,
    Exists,
    NotExists,
    In,
    NotIn,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QuerySortItem {
    field: String,
    direction: QuerySortDirection,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum QuerySortDirection {
    #[serde(rename = "1")]
    Ascending,
    #[serde(rename = "-1")]
    Descending,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryLimit {
    limit: Option<u32>,
    offset: Option<u32>,
}
