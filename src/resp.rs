
use async_std::io::prelude::BufReadExt;

#[derive(Debug)]
pub enum Resp {
    Simple(Vec<u8>),
    Error(Vec<u8>),
    Integer(i64),
    Bulk(Vec<u8>),
    Array(Vec<Resp>),
    Null,
}

impl Resp {

    pub async fn parse(mut reader: impl BufReadExt + std::marker::Unpin) -> anyhow::Result<Resp> 
    {
        let mut buf = Vec::new();
        let size = reader.read_until(b'\n', &mut buf).await?;
        if size == 0 {
            return Ok(Resp::Null);
        }
        match buf[0] {
            b'+' => {
                Ok(Resp::Simple(buf[1..buf.len()-2].to_vec()))
            },
            b'-' => {
                Ok(Resp::Error(buf[1..buf.len()-2].to_vec()))
            },
            b':' => {
                let n = atoi::atoi(&buf[1..buf.len()-2]).unwrap_or(0);
                Ok(Resp::Integer(n))
            },
            b'$' => {
                if b'-' == buf[1] {
                    return Ok(Resp::Null);
                }
                else {
                    let mut bulk_buf = Vec::new();
                    let size = reader.read_until(b'\n', &mut bulk_buf).await?;
                    if size == 0 {
                        return Ok(Resp::Null);
                    }
                    Ok(Resp::Bulk(bulk_buf[0..bulk_buf.len() -2].to_vec()))
                }
            },
            b'*' => {
                let len = atoi::atoi(&buf[1..buf.len()-2]).unwrap_or(0);
                let mut vec = Vec::new();
                for _ in 0..len {
                    let mut bulk_buf = Vec::new();
                    let size = reader.read_until(b'\n', &mut bulk_buf).await?;
                    if size == 0 {
                        vec.push(Resp::Null);
                        continue;
                    }
                    if b'-' == buf[1] {
                        vec.push(Resp::Null);
                        continue;
                    }
                    else {
                        let mut bulk_buf = Vec::new();
                        let size = reader.read_until(b'\n', &mut bulk_buf).await?;
                        if size == 0 {
                            vec.push(Resp::Null);
                            continue;
                        }
                        vec.push(Resp::Bulk(bulk_buf[0..bulk_buf.len() - 2].to_vec()));
                    }
                }
                Ok(Resp::Array(vec))
            },
            _ => Ok(Resp::Null)
        }
    }
}