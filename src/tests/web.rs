use anyhow::Result;
use std::collections::HashMap;

use super::{get_employees, Employee};
use rustmix::web::{
    http::Response,
    url::{self, AsUrl},
    *,
};

pub fn test_url() -> Result<()> {
    println!("\nTesting Url functions...");

    let url = "https://www.rust-lang.org".as_url()?;
    println!("Absolute URL: {}", &url);

    let url = ("https://www.rust-lang.org", "en-US", "documentation").as_url()?;
    println!("Absolute URL from parts: {}", &url);

    let url = "/path/to/relative/url".as_url()?;
    println!("Relative URL {}", &url);

    Ok(())
}

pub async fn test_reqwest() -> Result<()> {
    const BASE_URL: &str = "https://httpbin.org";

    println!("\nTesting reqwest functions...");
    println!("baseUrl: {BASE_URL}");

    let client = build_client().build()?;

    let url = (BASE_URL, "get?p1=foo&p2=baz").as_url()?;
    println!("Get: '{url}'");
    let response = client.get(url).send().await?;
    println!("response: {response:#?}");
    println!("json: {:#?}", response.json::<Response>().await?);

    let url = (BASE_URL, "post").as_url()?;
    let body = get_employees(3);
    println!("Post: '{url}'");
    let response = client
        .post(url)
        .json(&body)
        .send()
        .await?
        .json::<Response>()
        .await?;
    println!("response: {response:#?}");

    if let Some(data) = response.data {
        let employees: Vec<Employee> = serde_json::from_str(&data)?;
        println!("employees: {:#?}", employees);
    }

    let url = (BASE_URL, "ip").as_url()?;
    println!("IP: '{url}'");
    let response = client
        .get(url)
        .send()
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    println!("response: {response:#?}");

    let url = (BASE_URL, "cookies/set?freeform=test&ff=12345").as_url()?;
    println!("Cookies: '{url}'");
    let response = client
        .get(url)
        .send()
        .await?
        .json::<Response>()
        //.text()
        .await?;
    println!("response: {response:#?}");

    Ok(())
}
