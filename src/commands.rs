use crate::resp::Resp;

/// redis命令配置
pub struct Command<'a>(pub &'a [u8], pub i8);

/// redis命令
/// name 命令名
/// key 键
/// args 命令参数
pub struct CommandInfo {
    pub name: Vec<u8>,
    pub key: String,
    pub args: Vec<Vec<u8>>,
}

impl CommandInfo {

    /// 创建redis命令
    pub fn new() -> Self {
        CommandInfo {
            name: Vec::new(),
            key: String::new(),
            args: Vec::new(),
        }
    }

    /// 在命令表中找redis命令
    pub fn find_command(name: &[u8]) -> anyhow::Result<Command> {
        let cmd_tables = vec![
            Command(b"ping", 1),
            Command(b"set", 3),
            Command(b"get", 2),
            Command(b"del", 2),
            Command(b"lpush", 3),
            Command(b"rpop", 2),
        ];
        for command in cmd_tables {
            if name == command.0 {
                return Ok(command);
            }
        }
        Err(anyhow::anyhow!("ERR unknown command '{}'", CommandInfo::slice_to_string(name)))
    }

    /// redis协议Resp转命令
    pub fn from_resp(resp: Resp) -> anyhow::Result<Self> {
        if let Resp::Array(bulks) = resp {
            if bulks.is_empty() {
                return Err(anyhow::anyhow!("ERR Protocol error"));
            }
            if let Resp::Bulk(name) = &bulks[0] {
                let command = CommandInfo::find_command(name)?;
                let len = command.1 as usize;
                if bulks.len() != len {
                    return Err(anyhow::anyhow!("ERR wrong number of arguments for '{}' command", CommandInfo::slice_to_string(name)));
                }
                let mut cmd = CommandInfo::new();
                cmd.name = name.to_vec();
                if len > 1 {
                    cmd.parse_bulks(bulks)?;
                }
                return Ok(cmd);
            }
        }
       Err(anyhow::anyhow!("ERR Protocol error"))
    }

    /// 处理批量参数
    pub fn parse_bulks (&mut self, bulks: Vec<Resp>) -> anyhow::Result<()> {
        for i in 1..bulks.len() {
            let bulk;
            if let Resp::Bulk(t) = &bulks[i] {
                bulk = t;
            }  else  {
                return Err(anyhow::anyhow!("ERR args type wrong"));
            }
            if i == 1 {
                self.key = String::from_utf8(bulk.to_vec()).or(Err(anyhow::anyhow!("ERR key type wrong")))?;
            }
            else {
                self.args.push(bulk.to_vec());
            }
        }
        Ok(())
    }

    fn slice_to_string(slice: &[u8]) -> String {
        String::from_utf8(slice.to_vec()).unwrap_or("".to_string())
    }
}
