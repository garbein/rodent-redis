use async_std::sync::Arc;
use async_std::sync::Mutex;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) struct Db {
    id: u8,
    kv: Arc<Mutex<HashMap<String, Obj>>>,
}

#[derive(Debug, Clone)]
struct Obj {
    item: Vec<u8>,
    items: Vec<Vec<u8>>,
}

impl Obj {

    pub(crate) fn new() -> Self {
        Obj {
            item: Vec::new(),
            items: Vec::new(),
        }
    }
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
        let mut obj = Obj::new();
        obj.item = value;
        kv.insert(key, obj);
    }

    pub(crate) async fn get(&self, key: String) -> Option<Vec<u8>> {
        let kv = self.kv.lock().await;
        kv.get(&key).map(|v| v.item.clone())
    }

    pub(crate) async fn push(&self, key: String, value: Vec<u8>) {
        let mut kv = self.kv.lock().await;
        if let Some(obj) = kv.get_mut(&key) {
            obj.items.push(value);
        }
        else {
            let mut obj = Obj::new();
            obj.items.push(value);
            kv.insert(key, obj);
        }
    }

    pub(crate) async fn pop(&self, key: String) -> Option<Vec<u8>> {
        let mut kv = self.kv.lock().await;
        if let Some(obj) = kv.get_mut(&key) {
            return obj.items.pop();
        }
        None
    }
}
