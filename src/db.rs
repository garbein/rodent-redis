use async_std::sync::Arc;
use async_std::sync::Mutex;
use std::collections::{HashMap, VecDeque};
use crate::commands::CommandInfo;
use crate::resp::Resp;

/// 数据库
/// 用原子引用计数Arc使得db在线程间共享
/// 用互斥锁mutex可以安全并发
#[derive(Debug, Clone)]
pub struct Db {
    /// 数据库id
    pub id: u8,
    /// 数据key-value
    kv: Arc<Mutex<HashMap<String, Obj>>>,
}

/// 数据对象
#[derive(Debug, Clone)]
struct Obj {
    item: Vec<u8>,
    items: VecDeque<Vec<u8>>,
}

impl Obj {

    /// 创建数据对象
    pub fn new() -> Self {
        Obj {
            item: Vec::new(),
            items: VecDeque::new(),
        }
    }
}

impl Db {

    /// 创建数据库
    pub fn new() -> Self {
        Db {
            id: 0u8,
            kv: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// 执行命令
    pub async fn execute(&self, cmd: CommandInfo) -> Resp {
        let k = &cmd.name[..];
        if k == b"ping" {
            return self.ping().await;
        }
        match k {
            b"set" => self.set(cmd).await,
            b"get" => self.get(cmd).await,
            b"lpush" => self.push(cmd).await,
            b"rpop" => self.pop(cmd).await,
            b"del" => self.del(cmd).await,
            _ => Resp::Null,
        }
    }

    /// ping命令
    pub async fn ping(&self) -> Resp {
        Resp::Simple(b"PONG".to_vec())
    }

    /// set命令
    pub async fn set(&self, cmd: CommandInfo) -> Resp {
        let mut kv = self.kv.lock().await;
        let mut obj = Obj::new();
        obj.item = cmd.args[0].clone();
        kv.insert(cmd.key, obj);
        Resp::Simple(b"OK".to_vec())
    }

    /// set命令
    pub async fn get(&self, cmd: CommandInfo) -> Resp {
        let kv = self.kv.lock().await;
        let v = kv.get(&cmd.key).map(|v| v.item.clone());
        match v {
            Some(t) => Resp::Bulk(t),
            None => Resp::Null,
        }
    }

    /// del命令
    pub async fn del(&self, cmd: CommandInfo) -> Resp {
        let mut kv = self.kv.lock().await;
        kv.remove(&cmd.key);
        Resp::Simple(b"OK".to_vec())
    }

    /// push命令
    pub async fn push(&self, cmd: CommandInfo) -> Resp {
        let mut kv = self.kv.lock().await;
        let v = cmd.args[0].clone();
        let mut len = 1;
        if let Some(obj) = kv.get_mut(&cmd.key) {
            obj.items.push_back(v);
            len = obj.items.len();
        }
        else {
            let mut obj = Obj::new();
            obj.items.push_back(v);
            kv.insert(cmd.key, obj);
        }
        Resp::Integer(len as i64)
    }

    /// pop命令
    pub async fn pop(&self, cmd: CommandInfo) -> Resp {
        let mut kv = self.kv.lock().await;
        if let Some(obj) = kv.get_mut(&cmd.key) {
            if let Some(v) = obj.items.pop_front() {
                return Resp::Bulk(v);
            }
        }
        Resp::Null
    }
}
