use serde_json::Value;
use tokio_postgres::types::Json;

use crate::query::{
    Query, QueryFilterCondition, QueryFilterFilter, QueryFilterItem, QueryFilterOperator,
};

pub fn to_sql(table: &str, query: &Option<Query>) -> anyhow::Result<(String, Vec<Value>)> {
    let Some(query) = query else {
        return Ok((format!("SELECT data FROM {}", table), vec![]));
    };

    let mut where_str = String::new();
    let mut where_values = vec![];
    if let Some(filter) = &query.filter {
        for filter_item in filter {
            match filter_item {
                QueryFilterItem::Filter(filter) => {
                    let (filter_str, value) = to_sql_filter(filter)?;
                    where_str.push_str(filter_str.as_str());
                    where_values.push(value);
                }
                QueryFilterItem::Condition(condition) => {
                    let condition_str = to_sql_condition(condition)?;
                    where_str.push_str(condition_str.as_str());
                }
            }
        }
    }

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
        table, where_str, limit_str
    );
    println!("sql = {}", sql);
    Ok((sql, where_values))
}

fn to_sql_filter(def: &QueryFilterFilter) -> anyhow::Result<(String, Value)> {
    let filter = &def.filter;

    match &filter.operator {
        QueryFilterOperator::Equals => Ok((
            format!("data->'{}' = $1", &filter.field),
            filter.value.clone(),
        )),
        operator => Err(anyhow::anyhow!(
            "operator '{:?}' not yet implemented",
            operator
        )),
    }
}

fn to_sql_condition(condition: &QueryFilterCondition) -> anyhow::Result<String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_sql() {
        // let json = r#"{"filter":[{"type":"filter","operation":"and","filter":{"field":"id","operator":"equals","value":2}},{"type":"filter","operation":"or","filter":{"field":"name","operator":"equals","value":"John"}},{"type":"condition","operation":"and","filter":[{"type":"condition","operation":"and","filter":[{"type":"filter","operation":"and","filter":{"field":"id","operator":"notEquals","value":1}},{"type":"filter","operation":"and","filter":{"field":"name","operator":"equals","value":"Felipe"}},{"type":"condition","operation":"and","filter":[{"type":"filter","operation":"and","filter":{"field":"age","operator":"greaterThan","value":18}}]}]}]}],"sort":null,"limit":null}"#;
        // let query = Query::from_json(json).unwrap();
        // let sql = to_sql(&query);
        // assert_eq!(sql, "SELECT * FROM users WHERE id = 2 OR name = 'John' AND (id != 1 AND name = 'Felipe' AND (age > 18))");
    }
}
