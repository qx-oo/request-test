use clap::{App, Arg};
use futures::future::join_all;
use prettytable::{cell, color, row, Attr, Cell, Row, Table};
use reqwest;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;
use std::sync::Arc;
use std::time::SystemTime;
use tokio;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    sync_list: Vec<Value>,
    async_list: Vec<Value>,
    headers: HashMap<String, String>,
}

fn get_config(filename: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let file = match File::open(filename) {
        Ok(f) => f,
        Err(_) => Err("文件不存在")?,
    };
    let reader = BufReader::new(file);
    let val: Config = match serde_json::from_reader(reader) {
        Ok(cfg) => cfg,
        Err(_) => Err("配置文件格式错误")?,
    };
    Ok(val)
}

fn build_headers(config: &Arc<Config>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for (key, val) in &config.headers {
        headers.insert(
            HeaderName::from_str(&key).unwrap(),
            HeaderValue::from_str(&val).unwrap(),
        );
    }
    // println!("=================");
    // println!("build header: {:?}", headers);
    // println!("=================");
    headers
}

async fn request(item: &Value, headers: HeaderMap) -> Result<Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = match item["url"] {
        Value::String(ref url) => url,
        _ => Err("url type error")?,
    };
    let start = SystemTime::now();
    let response = match item["method"] {
        Value::String(ref method) => match method.as_ref() {
            "get" => client.get(url).headers(headers).send().await?.json().await,
            "post" => {
                client
                    .post(url)
                    .headers(headers)
                    .json(&item["data"])
                    .send()
                    .await?
                    .json()
                    .await
            }
            _ => Err("method error")?,
        },
        _ => Err("method error")?,
    };
    let (_, status): (Value, String) = match response {
        Ok(res) => (res, "success".to_owned()),
        _ => (json!({}), "fail".to_owned()),
    };
    // Ok([
    //     ("request", item.clone()),
    //     ("dur", start.elapsed().unwrap().as_secs_f64()),
    //     ("status": status),
    // ].iter().collect())
    Ok(json!({
        "request": item.clone(),
        // "response": res,
        "dur": start.elapsed().unwrap().as_secs_f64(),
        "status": status
    }))
}

async fn async_request(config: Arc<Config>) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let mut request_list = Vec::new();
    for item in &config.async_list {
        request_list.push(request(item, build_headers(&config)))
    }
    let response = join_all(request_list).await;
    Ok(response.into_iter().map(|item| item.unwrap()).collect())
}

async fn sync_request(config: Arc<Config>) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let mut results: Vec<Value> = Vec::new();
    for item in &config.sync_list {
        let val = request(item, build_headers(&config)).await?;
        results.push(val);
    }
    Ok(results)
}

fn get_table(results: Vec<Value>, notify_dur: f64) -> Table {
    let mut table = Table::new();
    table.add_row(row!["URL", "Dur", "Status",]);
    for item in results.iter() {
        let dur: f64 = serde_json::from_value(item["dur"].clone()).unwrap();
        let mut line_color = color::GREEN;
        if dur > notify_dur {
            line_color = color::RED;
        }
        table.add_row(Row::new(vec![
            Cell::new(&format!("{}", item["request"]["url"])),
            Cell::new(&format!("{}", item["dur"])).with_style(Attr::ForegroundColor(line_color)),
            Cell::new(&format!("{}", item["status"])),
            Cell::new(&format!("{:#?}", item["request"])),
        ]));
    }
    table
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("Qxoo Program")
        .version("1.0")
        .author("shawn <q-x64@live.com>")
        .about("Start Request Test")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("Config File")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("time")
                .short("t")
                .long("time")
                .value_name("Notify Duration Time, Default 0.8")
                .takes_value(true),
        )
        .get_matches();
    let config = matches.value_of("config").unwrap_or("config.json");
    let config = Arc::new(get_config(config)?);
    println!("Start Request Test");
    let notify_dur = matches.value_of("time").unwrap_or("0.8");
    let notify_dur = match notify_dur.parse::<f64>() {
        Ok(_dur) => _dur,
        Err(_) => Err("提醒阀值参数错误")?,
    };

    let sync_start = SystemTime::now();
    let results = sync_request(config.clone()).await?;
    let sync_dur: f64 = sync_start.elapsed().unwrap().as_secs_f64();

    let async_start = SystemTime::now();
    let async_results = async_request(config.clone()).await?;
    let async_dur: f64 = async_start.elapsed().unwrap().as_secs_f64();

    println!("=================");
    println!("Sync results");
    let table = get_table(results, notify_dur);
    table.printstd();
    println!("Total Dur: {}", sync_dur);
    println!("=================");
    println!("=================");
    println!("Async results");
    let table = get_table(async_results, notify_dur);
    table.printstd();
    println!("Total Dur: {}", async_dur);
    println!("=================");
    Ok(())
}
