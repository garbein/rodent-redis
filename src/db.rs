use async_std::sync::Arc;
use async_std::sync::Mutex;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) struct Db {
    id: u8,
    kv: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

#[derive(Debug)]
struct Value {
    id: u64,
    data: Vec<u8>,
}

impl Db {

    pub(crate) fn new() -> Self {
        Db {
            id: 0u8,
            kv: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub(crate) async fn set(&self, key: String, value: Vec<u8>) {
        let mut kv = self.kv.lock().await;
        kv.insert(key, value);
    }

    pub(crate) async fn get(&self, key: String) -> Option<Vec<u8>> {
        let kv = self.kv.lock().await;
        kv.get(&key).map(|v| v.clone())
    }
}
