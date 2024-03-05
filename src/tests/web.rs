use colored::Colorize;
use fake::{faker::internet::en::UserAgent, Fake};
use std::{collections::HashMap, error::Error};

use rustmix::web::{http::Response, url};

use super::{get_employees, Employee};

pub fn test_url_func() -> Result<(), Box<dyn Error>> {
    println!("\n{}", "Testing Url functions...".magenta());

    let url = url::create("https://www.rust-lang.org")?;
    println!("Absolute URL: {}", &url);

    let url = url::create("https://www.rust-lang.org")?
        .join("en-US")?
        .join("documentation")?;
    println!("Absolute URL from parts: {}", &url);

    let url = url::create("/path/to/relative/url")?;
    println!("Relative URL {}", &url);

    Ok(())
}

pub async fn test_reqwest_func() -> Result<(), Box<dyn Error>> {
    const BASE_URL: &str = "https://httpbin.org";

    println!("\n{}", "Testing reqwest functions...".magenta());
    println!("baseUrl: {BASE_URL}");

    let user_agent: String = UserAgent().fake();
    let client = reqwest::Client::builder()
        .user_agent(user_agent)
        .cookie_store(true)
        //.proxy(Proxy::http("http://localhost:8080")?)
        .build()?;

    let url = BASE_URL.to_owned() + "/get?p1=foo&p2=baz";
    println!("Get: '{url}'");
    let response = client.get(url).send().await?;
    println!("response: {response:#?}");
    println!("json: {:#?}", response.json::<Response>().await?);

    let url = BASE_URL.to_owned() + "/post";
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

    let url = BASE_URL.to_owned() + "/ip";
    println!("IP: '{url}'");
    let response = client
        .get(url)
        .send()
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    println!("response: {response:#?}");

    let url = BASE_URL.to_owned() + "/cookies/set?freeform=test&ff=12345";
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
