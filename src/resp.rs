
use async_std::io::prelude::BufReadExt;

/// redis协议枚举
/// 请求和回复都使用
#[derive(Debug)]
pub enum Resp {
    /// + 单行回复
    Simple(Vec<u8>),
    /// - 错误回复
    Error(Vec<u8>),
    /// : 整数回复
    Integer(i64),
    /// $ 批量回复
    Bulk(Vec<u8>),
    /// * 多个批量回复
    Array(Vec<Resp>),
    /// 空
    Null,
}

impl Resp {

    /// 解析redis协议
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