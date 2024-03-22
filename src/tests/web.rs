use anyhow::Result;
use std::collections::HashMap;

use super::{get_employees, Employee};
use rustmix::web::{http::Response, url, *};

pub fn test_url() -> Result<()> {
    println!("\nTesting Url functions...");

    let url = url::create("https://www.rust-lang.org");
    println!("Absolute URL: {}", &url);

    let url = url::create("https://www.rust-lang.org")
        .join("en-US")?
        .join("documentation")?;
    println!("Absolute URL from parts: {}", &url);

    let url = url::create("/path/to/relative/url");
    println!("Relative URL {}", &url);

    Ok(())
}

pub async fn test_reqwest() -> Result<()> {
    const BASE_URL: &str = "https://httpbin.org";

    println!("\nTesting reqwest functions...");
    println!("baseUrl: {BASE_URL}");

    let client = build_client().build()?;

    let url = url::create(BASE_URL).join("get?p1=foo&p2=baz")?;
    println!("Get: '{url}'");
    let response = client.get(url).send().await?;
    println!("response: {response:#?}");
    println!("json: {:#?}", response.json::<Response>().await?);

    let url = url::create(BASE_URL).join("post")?;
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

    let url = url::create(BASE_URL).join("ip")?;
    println!("IP: '{url}'");
    let response = client
        .get(url)
        .send()
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    println!("response: {response:#?}");

    let url = url::create(BASE_URL).join("cookies/set?freeform=test&ff=12345")?;
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
