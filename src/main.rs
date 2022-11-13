// use serde_json::{json, Map, Value};
// use std::sync::Arc;

mod store;

fn main() {
    // store = Store::new(TestPersistence);
}

// type Data = Map<String, Value>;

// trait Persistence {
//     fn save(&self, data: &Data);
// }

// struct Mongo;

// impl Persistence for Mongo {
//     fn save(&self, data: &Data) {
//         println!("saving with Mongo: {:?}", data);
//     }
// }

// struct Postgres;

// impl Persistence for Postgres {
//     fn save(&self, data: &Data) {
//         println!("saving with Postgres: {:?}", data);
//     }
// }

// #[derive(Clone)]
// struct Store {
//     persistence: Arc<dyn Persistence + Sync>,
// }

// impl Store {
//     fn new(persistence: impl Persistence + Sync) -> Self {
//         Self {
//             persistence: Arc::new(persistence),
//         }
//     }
//     fn save(&self, data: &Data) {
//         self.persistence.save(data);
//     }
// }

// fn main() {
//     let data = json!({
//         "name": "John Doe",
//         "age": 43,
//         "phones": [
//             "+44 1234567",
//             "+44 2345678"
//         ]
//     })
//     .as_object()
//     .unwrap()
//     .clone();

//     // Instead of always passing the Persistence implementation when saving the data here:
//     let store = Store::new(Postgres);
//     // I'd like to create a version of store that would carry the Persistence implementation.
//     // Something like:
//     // ```
//     // let store = Store::new(Mongo);
//     // store.save(&data);
//     // let app = App::new(store);
//     // // call app methods without passing the persistence around
//     // ```
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     struct Mock;

//     impl Persistence for Mock {
//         fn save(&self, data: &Data) {
//             println!("saving with Mock: {:?}", data);
//         }
//     }

//     #[test]
//     fn test_store() {
//         let data = json!({
//             "name": "John Doe",
//             "age": 43,
//             "phones": [
//                 "+44 1234567",
//                 "+44 2345678"
//             ]
//         })
//         .as_object()
//         .unwrap()
//         .clone();

//         Store::save(Mock, &data);
//     }
// }
