use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::{identity::Identity, query::Query};

type Data = Value;

#[async_trait]
pub trait Persistence {
    async fn find(&mut self, collection: &str, query: Option<Query>) -> anyhow::Result<Vec<Data>>;

    async fn find_one(
        &mut self,
        collection: &str,
        query: Option<Query>,
    ) -> anyhow::Result<Option<Data>>;

    // fn create(&mut self, record: &Data) -> anyhow::Result<Data>;

    // fn update(&mut self, record: &Data) -> anyhow::Result<Data>;

    // fn load(&mut self, id: Value) -> anyhow::Result<Option<Data>>;
}

#[derive(Clone)]
pub struct Store {
    persistence: Arc<Mutex<dyn Persistence + Sync>>,
}

#[allow(dead_code)]
impl Store {
    fn new(persistence: impl Persistence + Sync + 'static) -> Self {
        Self {
            persistence: Arc::new(Mutex::new(persistence)),
        }
    }

    pub async fn get<T>(&mut self, id: Value) -> anyhow::Result<Option<T>>
    where
        T: DeserializeOwned + Collection + Identity,
    {
        let collection = T::name();
        let query = T::identity_query(id);

        let mut persistence = self.persistence.lock().unwrap();
        let data = persistence.find_one(&collection, Some(query)).await?;
        match data {
            Some(data) => Ok(Some(serde_json::from_value(data)?)),
            None => Ok(None),
        }
    }

    pub async fn find<T>(&mut self, query: Option<Query>) -> anyhow::Result<Vec<T>>
    where
        T: DeserializeOwned + Collection,
    {
        let mut persistence = self.persistence.lock().unwrap();
        let collection = T::name();
        let values = persistence.find(&collection, query).await?;

        let mut new: Vec<T> = vec![];
        for v in values.into_iter() {
            new.push(serde_json::from_value(v)?);
        }
        Ok(new)
    }

    pub async fn find_one<T>(&mut self, query: Option<Query>) -> anyhow::Result<Option<T>>
    where
        T: DeserializeOwned + Collection,
    {
        let mut persistence = self.persistence.lock().unwrap();
        let collection = T::name();
        let value = persistence.find_one(&collection, query).await?;
        match value {
            Some(value) => Ok(Some(serde_json::from_value(value)?)),
            None => Ok(None),
        }
    }
}

struct TestPersistence {
    records: HashMap<String, Vec<Data>>,
}

#[async_trait]
impl Persistence for TestPersistence {
    async fn find(&mut self, collection: &str, query: Option<Query>) -> anyhow::Result<Vec<Data>> {
        let records = self.records.get(collection).unwrap();
        match query {
            Some(query) => {
                let mut new: Vec<Data> = vec![];
                for record in records.into_iter() {
                    if query.matches(record)? {
                        new.push(record.clone());
                    }
                }
                Ok(new)
            }
            None => Ok(records.clone()),
        }
    }

    async fn find_one(
        &mut self,
        collection: &str,
        query: Option<Query>,
    ) -> anyhow::Result<Option<Data>> {
        let records = self.records.get(collection).unwrap().clone();

        let Some(query) = query else {
            return Ok(records.get(0).cloned());
        };

        let records = self.records.get(collection).unwrap().clone();
        for record in records.into_iter() {
            if query.matches(&record)? {
                return Ok(Some(record.clone()));
            }
        }

        Ok(None)
    }

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

impl Identity for User {
    fn identity_query(id: Value) -> Query {
        Query::builder().eq("id", id).build()
    }

    fn identity(&self) -> Value {
        todo!()
    }

    fn key(&self) -> &str {
        todo!()
    }

    fn id(&self) -> Value {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Product {
    id: String,
    name: String,
}

pub trait Collection {
    fn name() -> String;
}

impl Collection for User {
    fn name() -> String {
        "users".to_string()
    }
}

impl Collection for Product {
    fn name() -> String {
        "products".to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[tokio::test]
    async fn test_identity() -> anyhow::Result<()> {
        let user1 = serde_json::to_value(User {
            id: "123".to_string(),
            name: "John".to_string(),
        })?;
        let user2 = serde_json::to_value(User {
            id: "456".to_string(),
            name: "Jane".to_string(),
        })?;

        let mut records = HashMap::new();
        records.insert("users".to_string(), vec![user1, user2]);
        let persistence = TestPersistence { records };
        let mut store = Store::new(persistence);
        let user = store.get::<User>(Value::String("456".to_string())).await?;
        assert_eq!(user.unwrap().name, "Jane");

        Ok(())
    }

    #[tokio::test]
    async fn test_find() -> anyhow::Result<()> {
        let mut records = HashMap::new();
        let user1 = serde_json::to_value(User {
            id: "123".to_string(),
            name: "John".to_string(),
        })?;
        let user2 = serde_json::to_value(User {
            id: "456".to_string(),
            name: "Jane".to_string(),
        })?;
        let product1 = serde_json::to_value(Product {
            id: "456".to_string(),
            name: "Apple".to_string(),
        })?;
        let product2 = serde_json::to_value(Product {
            id: "789".to_string(),
            name: "Banana".to_string(),
        })?;
        records.insert("users".to_string(), vec![user1, user2]);
        records.insert("products".to_string(), vec![product1, product2]);
        let persistence = TestPersistence { records };
        let mut store = Store::new(persistence);
        let users: Vec<User> = store.find(None).await?;
        let products: Vec<Product> = store.find(None).await?;

        println!("{:?}", users);
        println!("{:?}", products);

        let user: Option<User> = store.find_one(None).await?;
        let product: Option<Product> = store.find_one(None).await?;

        println!("{:?}", user);
        println!("{:?}", product);

        Ok(())
    }
}
