use serde_json::Value;

use crate::query::{
    Query, QueryFilterCondition, QueryFilterFilter, QueryFilterItem, QueryFilterOperation,
    QueryFilterOperator,
};

pub fn to_sql(table: &str, query: &Option<Query>) -> anyhow::Result<(String, Vec<Value>)> {
    let Some(query) = query else {
        return Ok((format!("SELECT data FROM {}", table), vec![]));
    };

    let (where_str, where_values) = match &query.filter {
        Some(filters) => items_to_sql(&filters)?,
        None => (String::new(), vec![]),
    };

    let mut limit_str = String::new();
    if let Some(limit_def) = &query.limit {
        if let Some(limit) = limit_def.limit {
            limit_str.push_str(format!(" LIMIT {}", limit).as_str());
        }
        if let Some(offset) = limit_def.offset {
            limit_str.push_str(format!(" OFFSET {}", offset).as_str());
        }
    }

    let sql = format!(
        "SELECT data FROM {} WHERE {}{}",
        table,
        enumerate_placeholders(where_str.as_str()),
        limit_str
    );
    Ok((sql, where_values))
}

fn items_to_sql(items: &Vec<QueryFilterItem>) -> anyhow::Result<(String, Vec<Value>)> {
    let mut where_str = String::new();
    let mut where_values = vec![];
    for (i, filter_item) in items.iter().enumerate() {
        match filter_item {
            QueryFilterItem::Filter(filter) => {
                if i > 0 {
                    where_str.push_str(&format!(" {} ", operation_to_sql(&filter.operation)));
                }
                let (filter_str, value) = filter_to_sql(filter)?;
                where_str.push_str(filter_str.as_str());
                where_values.push(value);
            }
            QueryFilterItem::Condition(condition) => {
                if i > 0 {
                    where_str.push_str(&format!(" {} ", operation_to_sql(&condition.operation)));
                }
                let (condition_str, values) = to_sql_condition(condition)?;
                where_str.push_str(condition_str.as_str());
                where_values.extend(values);
            }
        }
    }

    Ok((where_str, where_values))
}

fn enumerate_placeholders(sql: &str) -> String {
    let mut sql = sql.to_string();
    let mut i = 1;
    while let Some(index) = sql.find('?') {
        sql.replace_range(index..index + 1, &format!("${}", i));
        i += 1;
    }
    sql
}

// fn filters_to_sql(items: &Vec<QueryFilterFilter>) -> anyhow::Result<(String, Vec<Value>)> {
//     let mut where_str = String::new();
//     let mut where_values = vec![];
//     for (i, item) in items.iter().enumerate() {
//         let (filter_str, value) = filter_to_sql(item)?;
//         if i > 0 {
//             where_str.push_str(&format!(" {} ", operation_to_sql(&item.operation)));
//         }
//         where_str.push_str(filter_str.as_str());
//         where_values.push(value);
//     }
//     Ok((where_str, where_values))
// }

fn operation_to_sql(oper: &QueryFilterOperation) -> String {
    match oper {
        QueryFilterOperation::And => "AND",
        QueryFilterOperation::Or => "OR",
        QueryFilterOperation::Not => "NOT",
    }
    .to_string()
}

fn filter_to_sql(def: &QueryFilterFilter) -> anyhow::Result<(String, Value)> {
    let filter = &def.filter;

    match &filter.operator {
        QueryFilterOperator::Equals => Ok((
            format!("data->'{}' = ?", &filter.field),
            filter.value.clone(),
        )),
        QueryFilterOperator::NotEquals => Ok((
            format!("data->'{}' <> ?", &filter.field),
            filter.value.clone(),
        )),
        QueryFilterOperator::GreaterThan => Ok((
            format!("data->'{}' > ?", &filter.field),
            filter.value.clone(),
        )),
        operator => Err(anyhow::anyhow!(
            "operator '{:?}' not yet implemented",
            operator
        )),
    }
}

fn to_sql_condition(condition: &QueryFilterCondition) -> anyhow::Result<(String, Vec<Value>)> {
    let (where_str, params) = items_to_sql(&condition.filter)?;
    Ok((format!("({})", where_str), params))
}

#[cfg(test)]
mod tests {
    use crate::query::{QueryFilter, QueryFilterOperation};

    use super::*;

    #[test]
    fn test_enumerate_placeholders() {
        let str = "VALUES (?, ?)";
        assert_eq!(enumerate_placeholders(str), "VALUES ($1, $2)");
    }

    #[test]
    fn test_items_to_sql() {
        let items = vec![
            QueryFilterItem::Filter(QueryFilterFilter {
                operation: QueryFilterOperation::And,
                filter: QueryFilter {
                    field: "id".to_string(),
                    operator: QueryFilterOperator::Equals,
                    value: Value::String("123".to_string()),
                },
            }),
            QueryFilterItem::Filter(QueryFilterFilter {
                operation: QueryFilterOperation::Or,
                filter: QueryFilter {
                    field: "id".to_string(),
                    operator: QueryFilterOperator::Equals,
                    value: Value::String("456".to_string()),
                },
            }),
        ];
        let (where_str, where_values) = items_to_sql(&items).unwrap();
        let sql = "data->'id' = ? OR data->'id' = ?";
        assert_eq!(where_str, sql);
        assert_eq!(
            where_values,
            vec![
                Value::String("123".to_string()),
                Value::String("456".to_string())
            ]
        );
    }

    #[test]
    fn test_items_with_condition_to_sql() {
        let items = vec![
            QueryFilterItem::Filter(QueryFilterFilter {
                operation: QueryFilterOperation::And,
                filter: QueryFilter {
                    field: "id".to_string(),
                    operator: QueryFilterOperator::Equals,
                    value: Value::String("123".to_string()),
                },
            }),
            QueryFilterItem::Condition(QueryFilterCondition {
                operation: QueryFilterOperation::And,
                filter: vec![
                    QueryFilterItem::Filter(QueryFilterFilter {
                        operation: QueryFilterOperation::And,
                        filter: QueryFilter {
                            field: "name".to_string(),
                            operator: QueryFilterOperator::Equals,
                            value: Value::String("test1".to_string()),
                        },
                    }),
                    QueryFilterItem::Filter(QueryFilterFilter {
                        operation: QueryFilterOperation::Or,
                        filter: QueryFilter {
                            field: "name".to_string(),
                            operator: QueryFilterOperator::Equals,
                            value: Value::String("test2".to_string()),
                        },
                    }),
                ],
            }),
        ];
        let (where_str, where_values) = items_to_sql(&items).unwrap();

        let sql = "data->'id' = ? AND (data->'name' = ? OR data->'name' = ?)";
        assert_eq!(sql, where_str);
        assert_eq!(where_values.len(), 3);
        assert_eq!(where_values[0], Value::String("123".to_string()));
        assert_eq!(where_values[1], Value::String("test1".to_string()));
        assert_eq!(where_values[2], Value::String("test2".to_string()));
    }

    #[test]
    fn test_to_sql() -> anyhow::Result<()> {
        let json = r#"{"filter":[{"type":"filter","operation":"and","filter":{"field":"id","operator":"equals","value":2}},{"type":"filter","operation":"or","filter":{"field":"name","operator":"equals","value":"John"}},{"type":"condition","operation":"and","filter":[{"type":"condition","operation":"and","filter":[{"type":"filter","operation":"and","filter":{"field":"id","operator":"notEquals","value":1}},{"type":"filter","operation":"and","filter":{"field":"name","operator":"equals","value":"Felipe"}},{"type":"condition","operation":"and","filter":[{"type":"filter","operation":"and","filter":{"field":"age","operator":"greaterThan","value":18}}]}]}]}],"sort":null,"limit":null}"#;
        let query = Query::from_json(json).unwrap();
        let (sql, params) = to_sql("users", &Some(query))?;
        assert_eq!(sql, "SELECT data FROM users WHERE data->'id' = $1 OR data->'name' = $2 AND ((data->'id' <> $3 AND data->'name' = $4 AND (data->'age' > $5)))");
        assert_eq!(params.len(), 5);
        assert_eq!(params[0], 2);
        assert_eq!(params[1], "John");
        assert_eq!(params[2], 1);
        assert_eq!(params[3], "Felipe");
        assert_eq!(params[4], 18);

        Ok(())
    }
}
