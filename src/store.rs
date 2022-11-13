use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

type Data = Value;

#[async_trait]
pub trait Persistence {
    async fn find(&mut self, query: Option<Value>) -> anyhow::Result<Vec<Data>>;

    // fn find_one(&mut self, query: Option<Value>) -> anyhow::Result<Option<Data>>;

    // fn create(&mut self, record: &Data) -> anyhow::Result<Data>;

    // fn update(&mut self, record: &Data) -> anyhow::Result<Data>;

    // fn load(&mut self, id: Value) -> anyhow::Result<Option<Data>>;
}

#[derive(Clone)]
pub struct Store {
    persistence: Arc<Mutex<dyn Persistence + Sync>>,
}

impl Store {
    fn new(persistence: impl Persistence + Sync + 'static) -> Self {
        Self {
            persistence: Arc::new(Mutex::new(persistence)),
        }
    }

    pub async fn find<T>(&mut self, id: &str) -> anyhow::Result<Vec<T>>
    where
        T: DeserializeOwned,
    {
        let mut persistence = self.persistence.lock().unwrap();
        let values = persistence.find(Some(serde_json::from_str(id)?)).await?;

        let mut new: Vec<T> = vec![];
        for v in values.into_iter() {
            new.push(serde_json::from_value(v)?);
        }
        Ok(new)
    }
}

struct TestPersistence {
    name: String,
}

#[async_trait]
impl Persistence for TestPersistence {
    async fn find(&mut self, query: Option<Value>) -> anyhow::Result<Vec<Data>> {
        println!("find");
        Ok(vec![])
    }
    // async fn find_one<T>(&mut self, query: Option<Value>) -> anyhow::Result<Option<T>> {
    //     println!("find_one");
    //     Ok(None)
    // }
    // async fn create<T>(&mut self, record: &T) -> anyhow::Result<T> {
    //     println!("create");
    //     Ok(record.clone())
    // }
    // async fn update<T>(&mut self, record: &T) -> anyhow::Result<T> {
    //     println!("update");
    //     Ok(record.clone())
    // }
    // async fn load<T>(&mut self, id: Value) -> anyhow::Result<Option<T>> {
    //     println!("load");
    //     Ok(None)
    // }
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find() -> anyhow::Result<()> {
        let mut store = Store::new(TestPersistence {
            name: "Felipe".to_string(),
        });
        let user: Vec<User> = store.find("123").await?;
        println!("{:?}", user);
        Ok(())
    }
}
