use serde_json:: { json, Value };
use anyhow::{ anyhow, Result };


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
