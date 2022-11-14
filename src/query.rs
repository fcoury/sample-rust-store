use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Query {
    pub filter: Option<Vec<QueryFilterItem>>,
    pub sort: Option<Vec<QuerySortItem>>,
    pub limit: Option<QueryLimit>,
}

impl Query {
    pub fn builder() -> QueryBuilder {
        QueryBuilder::new()
    }

    #[allow(unused)]
    pub fn from_json(json: &str) -> anyhow::Result<Query> {
        serde_json::from_str(json)
            .context(format!("Failed to parse query from JSON string: {}", json))
    }

    pub fn matches(&self, value: &Value) -> anyhow::Result<bool> {
        if let Some(filter) = &self.filter {
            for filter_item in filter {
                if !filter_item.matches(value)? {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum QueryFilterItem {
    Filter(QueryFilterFilter),
    Condition(QueryFilterCondition),
}

impl QueryFilterItem {
    pub fn matches(&self, value: &Value) -> anyhow::Result<bool> {
        match self {
            QueryFilterItem::Filter(filter) => filter.matches(value),
            QueryFilterItem::Condition(condition) => condition.matches(value),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueryFilterFilter {
    pub operation: QueryFilterOperation,
    pub filter: QueryFilter,
}

impl QueryFilterFilter {
    pub fn matches(&self, value: &Value) -> anyhow::Result<bool> {
        self.filter.matches(value)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueryFilterCondition {
    pub operation: QueryFilterOperation,
    pub filter: Vec<QueryFilterItem>,
}

impl QueryFilterCondition {
    pub fn matches(&self, value: &Value) -> anyhow::Result<bool> {
        for filter_item in &self.filter {
            if !filter_item.matches(value)? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueryFilter {
    pub field: String,
    pub operator: QueryFilterOperator,
    pub value: Value,
}

impl QueryFilter {
    pub fn matches(&self, value: &Value) -> anyhow::Result<bool> {
        let field_value = value.get(&self.field).context(format!(
            "field '{}' not found on value: {:?}",
            &self.field, value
        ))?;

        match &self.operator {
            QueryFilterOperator::Equals => Ok(field_value == &self.value),
            QueryFilterOperator::NotEquals => Ok(field_value != &self.value),
            QueryFilterOperator::LessThan => {
                if let Value::Number(field_value) = field_value {
                    if let Value::Number(value) = &self.value {
                        let field_value = field_value.as_f64().context(format!(
                            "can't transform field value {:?} to f64",
                            field_value
                        ))?;
                        let value = value.as_f64().context(format!(
                            "can't transform comparison value {:?} to f64",
                            value
                        ))?;
                        Ok(field_value < value)
                    } else {
                        // false
                        Err(anyhow!(
                            "Error comparing values: {:?} < {:?}",
                            field_value,
                            value
                        ))
                    }
                } else {
                    Err(anyhow!(
                        "Error comparing values: {:?} < {:?}",
                        field_value,
                        value
                    ))
                }
            }
            // QueryFilterOperator::LessThanOrEqual => field_value <= &self.value,
            // QueryFilterOperator::GreaterThan => field_value > &self.value,
            // QueryFilterOperator::GreaterThanOrEqual => field_value >= &self.value,
            QueryFilterOperator::In => {
                if let Value::Array(array) = &self.value {
                    Ok(array.contains(field_value))
                } else {
                    Err(anyhow!(
                        "Error comparing values: {:?} in {:?}",
                        field_value,
                        self.value
                    ))
                }
            }
            QueryFilterOperator::NotIn => {
                if let Value::Array(array) = &self.value {
                    Ok(!array.contains(field_value))
                } else {
                    Err(anyhow!(
                        "Error comparing values: {:?} not in {:?}",
                        field_value,
                        self.value
                    ))
                }
            }
            operator => unimplemented!("operator {:?} not implemented", operator),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum QueryFilterOperation {
    And,
    Or,
    Not,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuerySortItem {
    field: String,
    direction: QuerySortDirection,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum QuerySortDirection {
    #[serde(rename = "1")]
    Ascending,
    #[serde(rename = "-1")]
    Descending,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueryLimit {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct QueryBuilder {
    filter: Vec<QueryFilterItem>,
}

#[allow(dead_code)]
impl QueryBuilder {
    pub fn new() -> QueryBuilder {
        QueryBuilder::default()
    }

    pub fn wher(&mut self, field: &str, value: Value) -> &mut QueryBuilder {
        self.filter.push(QueryFilterItem::Filter(QueryFilterFilter {
            operation: QueryFilterOperation::And,
            filter: QueryFilter {
                field: field.to_string(),
                operator: QueryFilterOperator::Equals,
                value,
            },
        }));
        self
    }

    pub fn or_wher(&mut self, field: &str, value: Value) -> &mut QueryBuilder {
        self.filter.push(QueryFilterItem::Filter(QueryFilterFilter {
            operation: QueryFilterOperation::Or,
            filter: QueryFilter {
                field: field.to_string(),
                operator: QueryFilterOperator::Equals,
                value,
            },
        }));
        self
    }

    pub fn and_wher(&mut self, field: &str, value: Value) -> &mut QueryBuilder {
        self.wher(field, value)
    }

    pub fn and<F>(&mut self, mut query_fn: F) -> &mut QueryBuilder
    where
        F: FnMut(QueryBuilder) -> Query,
    {
        let query_builder = QueryBuilder::new();
        let query = query_fn(query_builder);
        self.filter
            .push(QueryFilterItem::Condition(QueryFilterCondition {
                operation: QueryFilterOperation::And,
                filter: query.filter.unwrap(),
            }));
        self
    }

    pub fn eq(&mut self, field: &str, value: Value) -> &mut QueryBuilder {
        self.filter.push(QueryFilterItem::Filter(QueryFilterFilter {
            operation: QueryFilterOperation::And,
            filter: QueryFilter {
                field: field.to_string(),
                operator: QueryFilterOperator::Equals,
                value,
            },
        }));
        self
    }

    pub fn not_eq(&mut self, field: &str, value: Value) -> &mut QueryBuilder {
        self.filter.push(QueryFilterItem::Filter(QueryFilterFilter {
            operation: QueryFilterOperation::And,
            filter: QueryFilter {
                field: field.to_string(),
                operator: QueryFilterOperator::NotEquals,
                value,
            },
        }));
        self
    }

    pub fn gt(&mut self, field: &str, value: Value) -> &mut QueryBuilder {
        self.filter.push(QueryFilterItem::Filter(QueryFilterFilter {
            operation: QueryFilterOperation::And,
            filter: QueryFilter {
                field: field.to_string(),
                operator: QueryFilterOperator::GreaterThan,
                value,
            },
        }));
        self
    }

    pub fn build(&self) -> Query {
        Query {
            filter: Some(self.filter.clone()),
            sort: None,
            limit: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_query_builder() {
        let query = Query::builder()
            .wher("id", json!(2))
            .or_wher("name", json!("John"))
            .and(|mut q| {
                q.and(|mut q| {
                    q.not_eq("id", json!(1))
                        .eq("name", json!("Felipe"))
                        .and(|mut q| q.gt("age", json!(18)).build())
                        .build()
                })
                .build()
            })
            .build();

        // FIXME improve the test like test_query
        let json = serde_json::to_string(&query).unwrap();
        let expected = r#"{"filter":[{"type":"filter","operation":"and","filter":{"field":"id","operator":"equals","value":2}},{"type":"filter","operation":"or","filter":{"field":"name","operator":"equals","value":"John"}},{"type":"condition","operation":"and","filter":[{"type":"condition","operation":"and","filter":[{"type":"filter","operation":"and","filter":{"field":"id","operator":"notEquals","value":1}},{"type":"filter","operation":"and","filter":{"field":"name","operator":"equals","value":"Felipe"}},{"type":"condition","operation":"and","filter":[{"type":"filter","operation":"and","filter":{"field":"age","operator":"greaterThan","value":18}}]}]}]}],"sort":null,"limit":null}"#;
        assert_eq!(json, expected);
    }

    #[test]
    fn test_query() {
        let query = Query {
            filter: Some(vec![QueryFilterItem::Filter(QueryFilterFilter {
                operation: QueryFilterOperation::And,
                filter: QueryFilter {
                    field: "name".to_string(),
                    operator: QueryFilterOperator::Equals,
                    value: Value::String("test".to_string()),
                },
            })]),
            sort: Some(vec![QuerySortItem {
                field: "name".to_string(),
                direction: QuerySortDirection::Ascending,
            }]),
            limit: Some(QueryLimit {
                limit: Some(10),
                offset: Some(30),
            }),
        };

        let json = serde_json::to_value(&query).unwrap();
        assert_eq!(json["filter"].as_array().unwrap().len(), 1);

        let filter = json["filter"][0].clone();
        assert_eq!(filter["type"], "filter");
        assert_eq!(filter["operation"], "and");

        let filter = filter["filter"].clone();
        assert_eq!(filter["field"], "name");
        assert_eq!(filter["operator"], "equals");
        assert_eq!(filter["value"], "test");
    }

    #[test]
    fn complex_query() {
        let query = Query {
            filter: Some(vec![QueryFilterItem::Condition(QueryFilterCondition {
                operation: QueryFilterOperation::And,
                filter: vec![
                    QueryFilterItem::Filter(QueryFilterFilter {
                        operation: QueryFilterOperation::And,
                        filter: QueryFilter {
                            field: "id".to_string(),
                            operator: QueryFilterOperator::NotEquals,
                            value: Value::Number(1.into()),
                        },
                    }),
                    QueryFilterItem::Filter(QueryFilterFilter {
                        operation: QueryFilterOperation::And,
                        filter: QueryFilter {
                            field: "age".to_string(),
                            operator: QueryFilterOperator::GreaterThan,
                            value: Value::Number(18.into()),
                        },
                    }),
                ],
            })]),
            sort: Some(vec![QuerySortItem {
                field: "name".to_string(),
                direction: QuerySortDirection::Ascending,
            }]),
            limit: Some(QueryLimit {
                limit: Some(10),
                offset: Some(30),
            }),
        };

        // FIXME improve the test like test_query
        let expected = r#"{"filter":[{"type":"condition","operation":"and","filter":[{"type":"filter","operation":"and","filter":{"field":"id","operator":"notEquals","value":1}},{"type":"filter","operation":"and","filter":{"field":"age","operator":"greaterThan","value":18}}]}],"sort":[{"field":"name","direction":"1"}],"limit":{"limit":10,"offset":30}}"#;
        let json = serde_json::to_string(&query).unwrap();
        assert_eq!(json, expected);
    }
}
