use crate::resp::Resp;

pub struct Command {
    name: Vec<u8>,
    key: Vec<u8>,
    args: Vec<Vec<u8>>,
}

impl Command {

    pub fn new() -> Self {
        Command {
            name: Vec::new(),
            key: Vec::new(),
            args: Vec::new(),
        }
    }

    pub fn from_resp(resp: Resp) -> anyhow::Result<Self> {
        match resp {
            Resp::Array(bulks) => {
                if let Resp::Bulk(vec) = &bulks[0] {
                    let mut cmd = Command::new();
                    cmd.name = vec[..].to_vec();
                    match &vec[..] {
                        b"ping" => {

                        }
                        b"set" => {

                           
                        }
                        b"get" => { 
                           
                        }
                        b"lpush" => {
                          
                        }
                        b"rpop" => {
                            
                        }
                        _ => {
                            return Err(anyhow::anyhow!("ERR unknown command `{}`"));
                        }
                    }
                    return Ok(cmd);
                }
            }
            _ => {

            }
        }
       Err(anyhow::anyhow!("ERR Protocol error"))
    }
}
