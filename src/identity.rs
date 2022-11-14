use serde_json::Value;

use crate::query::Query;

pub trait Identity {
    fn identity_query(id: Value) -> Query;

    /// The attribute that uniquely identifies the object when serialized.
    ///
    /// # Example
    ///
    /// ```
    /// #[derive(Serialize, Deserialize)]
    /// struct User {
    ///     #[serde(rename = "_id")]
    ///     id: String,
    ///     name: String,
    /// }
    ///
    /// impl Identity for User {
    ///     fn identity() -> &'static str {
    ///         "_id"
    ///     }
    /// }
    /// ```
    fn key(&self) -> &str;

    /// The value of the identity attribute.
    ///
    /// # Example
    ///
    /// ```
    /// #[derive(Serialize, Deserialize)]
    /// struct User {
    ///    #[serde(rename = "_id")]
    ///    id: String,
    ///    name: String,
    /// }
    ///
    /// impl Identity for User {
    ///    fn id() -> &'static str {
    ///        "_id"
    ///    }
    /// }
    fn id(&self) -> Value;

    /// The representation of how the identity key-pair should be.
    ///
    /// # Example
    ///
    /// ```
    /// #[derive(Serialize, Deserialize)]
    /// struct User {
    ///    #[serde(rename = "_id")]
    ///   id: String,
    ///   name: String,
    /// }
    ///
    /// impl Identity for User {
    ///    fn identity() -> Value {
    ///       json!({
    ///         "_id": self.id
    ///       })
    ///    }
    /// }
    fn identity(&self) -> Value;
}
