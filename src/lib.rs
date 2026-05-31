use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;

use serde_json:: { json, Value };
use anyhow::{ anyhow, Result };

pub mod mail;

pub fn print_json(value: &Value) {
    println!("{}", serde_json::to_string_pretty(value).unwrap())
}

pub fn read_json(file: &str, key: &str) -> Result<Value> {
    let file = match std::fs::File::open(file) {
        Ok(resp) => resp,
        Err(e) => return Err(anyhow!(e))
    };
    match serde_json::from_reader::<_, Value>(file) {
        Ok(resp) => {
            match resp.get(key) {
                Some(result) => Ok(json!(result)),
                None => Err(anyhow!("字段不存在"))
            }
        }
        Err(e) => Err(anyhow!("json解析失败：{}", e))
    }
}

pub fn file_to_json(file: &str) -> Result<Value> {
    let file = match std::fs::File::open(file) {
        Ok(resp) => resp,
        Err(e) => return Err(anyhow!(e))
    };
    match serde_json::from_reader::<_, Value>(file) {
        Ok(resp) => Ok(resp),
        Err(e) => Err(anyhow!("json解析失败：{}", e))
    }
}

pub fn json_to_file(file: &str, content: &Value) -> Result<()>{
    let file = match std::fs::File::create(file) {
        Ok(resp) => resp,
        Err(e) => return Err(anyhow!("文件不存在或无权限：{}", e)),
    };
    serde_json::to_writer_pretty(file, content)?;
    Ok(())
}

pub fn is_socket_in_use<P: AsRef<Path>>(path: P) -> bool {
    match UnixStream::connect(path) {
        Ok(_) => true,      // 连接成功，说明有服务端在监听
        Err(e) => {
            match e.raw_os_error() {
                Some(111) => false, // ECONNREFUSED: 没有进程在监听
                Some(2) => false,   // ENOENT: 文件不存在
                _ => true,          // 其他错误（权限等）视为可能在使用
            }
        }
    }
}

pub fn bind_socket<P: AsRef<Path>>(path: P) -> Result<UnixListener> {
    let path = path.as_ref();
    if path.exists() {
        if is_socket_in_use(path) {
            return Err(anyhow!("socket {:?} 正在被其他进程使用", path));
        } else {
            std::fs::remove_file(path)?;
        }
    }
    Ok(UnixListener::bind(path)?)
}
