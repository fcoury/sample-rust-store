// use async_trait::async_trait;
// use serde::Serialize;
// use serde::{de::DeserializeOwned, Deserialize};
// use serde_json::{json, Value};

// pub trait Identity {
//     /// The attribute that uniquely identifies the object when serialized.
//     ///
//     /// # Example
//     ///
//     /// ```
//     /// #[derive(Serialize, Deserialize)]
//     /// struct User {
//     ///     #[serde(rename = "_id")]
//     ///     id: String,
//     ///     name: String,
//     /// }
//     ///
//     /// impl Identity for User {
//     ///     fn identity() -> &'static str {
//     ///         "_id"
//     ///     }
//     /// }
//     /// ```
//     fn key(&self) -> &str;

//     /// The value of the identity attribute.
//     ///
//     /// # Example
//     ///
//     /// ```
//     /// #[derive(Serialize, Deserialize)]
//     /// struct User {
//     ///    #[serde(rename = "_id")]
//     ///    id: String,
//     ///    name: String,
//     /// }
//     ///
//     /// impl Identity for User {
//     ///    fn id() -> &'static str {
//     ///        "_id"
//     ///    }
//     /// }
//     fn id(&self) -> Value;

//     /// The representation of how the identity key-pair should be.
//     ///
//     /// # Example
//     ///
//     /// ```
//     /// #[derive(Serialize, Deserialize)]
//     /// struct User {
//     ///    #[serde(rename = "_id")]
//     ///   id: String,
//     ///   name: String,
//     /// }
//     ///
//     /// impl Identity for User {
//     ///    fn identity() -> Value {
//     ///       json!({
//     ///         "_id": self.id
//     ///       })
//     ///    }
//     /// }
//     fn identity(&self) -> Value;
// }

// #[async_trait]
// pub trait Strategy<R>
// where
//     R: Record,
// {
//     async fn load(id: Value) -> anyhow::Result<Self>
//     where
//         Self: Sized;
// }

// #[async_trait]
// pub trait RecordWithStrategy<R, S>
// where
//     R: Record,
//     S: Strategy<R>,
// {
//     async fn load(id: Value) -> anyhow::Result<R>
//     where
//         Self: Sized;
// }

// #[async_trait]
// pub trait Record: Sized + DeserializeOwned + Serialize + Identity {
//     async fn load(id: Value) -> anyhow::Result<Self>
//     where
//         Self: Sized;
//     async fn create(data: Self) -> anyhow::Result<Self>;
//     // async fn update(&self, data: Box<T>) -> anyhow::Result<()>;
//     // async fn delete(&self, id: Value) -> anyhow::Result<()>;
//     // async fn find(&self, query: Option<Value>) -> anyhow::Result<Vec<T>>;
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct User {
//     id: String,
//     name: String,
// }

// impl Identity for User {
//     fn key(&self) -> &str {
//         "_id"
//     }

//     fn id(&self) -> Value {
//         json!(self.id)
//     }

//     fn identity(&self) -> Value {
//         json!({
//             "_id": self.id
//         })
//     }
// }

// #[async_trait]
// impl Record for User {
//     async fn load(id: Value) -> anyhow::Result<Self> {
//         Ok(User {
//             id: id.to_string(),
//             name: "John".to_string(),
//         })
//     }

//     async fn create(data: Self) -> anyhow::Result<Self> {
//         Ok(data)
//     }
// }

// #[tokio::main]
// async fn main() {
//     let user = User {
//         id: "123".to_string(),
//         name: "John".to_string(),
//     };

//     println!("{:?}", user.identity());
//     let user = User::load(json!("123")).await.unwrap();
//     println!("{:?}", user);
//     let user = User::create(user).await.unwrap();
//     println!("{:?}", user);
// }
