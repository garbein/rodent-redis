use async_std::sync::Arc;
use async_std::sync::Mutex;
use std::collections::{HashMap, VecDeque};
use crate::commands::CommandInfo;
use crate::resp::Resp::{self, *};

#[derive(Debug, Clone)]
pub struct Db {
    id: u8,
    kv: Arc<Mutex<HashMap<String, Obj>>>,
}

#[derive(Debug, Clone)]
struct Obj {
    item: Vec<u8>,
    items: VecDeque<Vec<u8>>,
}

impl Obj {

    pub fn new() -> Self {
        Obj {
            item: Vec::new(),
            items: VecDeque::new(),
        }
    }
}

impl Db {

    pub fn new() -> Self {
        Db {
            id: 0u8,
            kv: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub async fn execute(&self, cmd: CommandInfo) -> Resp {
        let k = &cmd.name[..];
        if k == b"ping" {
            return self.ping().await;
        }
        match k {
            b"set" => self.set(cmd).await,
            b"get" => self.get(cmd).await,
            _ => Null,
        };
        Simple(b"PONG".to_vec())
    }

    pub async fn ping(&self) -> Resp {
        Simple(b"PONG".to_vec())
    }

    pub async fn set(&self, cmd: CommandInfo) -> Resp {
        let mut kv = self.kv.lock().await;
        let mut obj = Obj::new();
        obj.item = cmd.args[0].clone();
        kv.insert(cmd.key, obj);
        Simple(b"OK".to_vec())
    }

    pub async fn get(&self, cmd: CommandInfo) -> Resp {
        let kv = self.kv.lock().await;
        let v = kv.get(&cmd.key).map(|v| v.item.clone());
        match v {
            Some(t) => Resp::Bulk(t),
            None => Resp::Null,
        }
    }

    pub async fn push(&self, key: String, value: Vec<u8>) {
        let mut kv = self.kv.lock().await;
        if let Some(obj) = kv.get_mut(&key) {
            obj.items.push_back(value);
        }
        else {
            let mut obj = Obj::new();
            obj.items.push_back(value);
            kv.insert(key, obj);
        }
    }

    pub async fn pop(&self, key: String) -> Option<Vec<u8>> {
        let mut kv = self.kv.lock().await;
        if let Some(obj) = kv.get_mut(&key) {
            return obj.items.pop_front();
        }
        None
    }
}
