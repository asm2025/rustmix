use colored::Colorize;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    io::{LineWriter, Write},
    path::PathBuf,
};

use asmtools::{
    io::{
        directory,
        file::{self, FileEx, FileOpenOptions},
        path,
    },
    web::url,
};

#[derive(Debug, Serialize, Deserialize)]
struct Person {
    name: String,
    age: u8,
    email: Option<String>,
    phones: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // test_path_func()?;
    // test_directory_func()?;
    // test_file_func()?;
    // test_url_func()?;
    test_reqwest_func().await?;
    Ok(())
}

fn test_path_func() -> Result<(), Box<dyn Error>> {
    println!("\n{}", "Testing path functions...".magenta());

    let path = path::create(r"C:\").join("MyFile.txt");
    println!("{}", path.display());

    let path = path::create(r"C:\").join("My Folder").join("MyFile.txt");
    println!("{}", path.display());

    let path = path::create(r"C:\")
        .join("My Folder")
        .join("My Subfolder")
        .join("MyFile.txt");
    println!("{}", path.display());

    let path: PathBuf = [r"C:\", "My Folder", "My Subfolder", "", "NonEmpty"]
        .iter()
        .collect();
    println!("{}", path.display());
    Ok(())
}

fn test_directory_func() -> Result<(), Box<dyn Error>> {
    println!("\n{}", "Testing directory functions...".magenta());

    let curdir = directory::current();
    let original_path_len = curdir.components().count();
    let path = path::create(curdir.to_str().unwrap())
        .join("My Folder")
        .join("My Subfolder")
        .join("NonEmpty");

    println!(
        "Current Dir: '{}'. It has {} components.",
        curdir.display().to_string().green(),
        original_path_len.to_string().cyan()
    );

    let exists = directory::exists(&path);
    println!(
        "Does the directory exist? {}",
        if exists { "Yes".green() } else { "No".red() }
    );

    if !exists {
        println!(
            "\nI will try to create the directory '{}'",
            path.display().to_string().yellow()
        );

        let created = match directory::create(&path) {
            Ok(_) => None,
            Err(e) => Some(format!("Error: {}", e)),
        };

        if created.is_none() {
            println!("{}", "Directory created using create()".underline());
        } else {
            println!("{}", created.unwrap().yellow());
        }
    }

    println!("\n{}", "I will delete the directory.".magenta());
    let path = path::take(&path, original_path_len);
    println!("The path is now '{}'", path.display().to_string().green());
    match directory::delete(&path) {
        Ok(_) => println!("{}", "Directory deleted.".green()),
        Err(e) => println!("Error: {}", e.to_string().red()),
    }
    Ok(())
}

fn test_file_func() -> Result<(), Box<dyn Error>> {
    println!("\n{}", "Testing file functions...".magenta());

    let curdir = directory::current();
    let original_path_len = curdir.components().count();
    let mut path = path::create(curdir.to_str().unwrap())
        .join("My Folder")
        .join("My Subfolder")
        .join("NonEmpty");

    println!(
        "\n\nCurrent Dir: '{}'. It has {} components.",
        curdir.display().to_string().green(),
        original_path_len.to_string().cyan()
    );

    let exists = path::exists(&path);
    println!(
        "Does the file exist? {}",
        if exists { "Yes".green() } else { "No".red() }
    );

    println!(
        "I will create or open the file '{}'",
        &path.display().to_string().yellow()
    );
    let file = file::create(&path, Some(FileOpenOptions::Default))?;
    println!("{}", "File created or openned.".green());

    let mut writer = LineWriter::new(file);
    writeln!(&mut writer, "Hello, world!")?;
    writeln!(
        &mut writer,
        "The next line will be filtered out when the filter is applied."
    )?;
    writeln!(&mut writer, "!12345!")?;

    for _ in 0..20 {
        let random_string: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(9)
            .map(char::from)
            .collect();
        writeln!(&mut writer, "{}", random_string)?;
    }

    println!("File written.");
    drop(writer);
    println!("File closed.");

    println!(
        "\n{} {}",
        "I will try to open the file '{}' and read it".magenta(),
        &path.display().to_string().yellow()
    );

    let file = file::open(&path)?;

    for line in file.read()? {
        println!("{}", line);
    }

    drop(file);

    println!("\n{}", "I will apply the filter now.".magenta());

    let file = file::open(&path)?;

    for line in file.read_filtered(|e: &str| !e.contains("12345"))? {
        println!("{}", line);
    }

    drop(file);

    println!(
        "\n{}{}{}",
        "I will open the file and read it in batches of ".magenta(),
        "5".cyan(),
        " lines.".magenta()
    );
    let file = file::open(&path)?;
    file.read_batch(5, |batch, lines| print_batch(batch, lines))?;
    drop(file);

    println!(
        "\n{}{}{}",
        "I will open the file and read it in batches of ".magenta(),
        "5".cyan(),
        " lines and apply the filter.".magenta()
    );
    let file = file::open(&path)?;
    file.read_batch_filtered(
        5,
        |e: &str| !e.contains("12345"),
        |batch, lines| print_batch(batch, lines),
    )?;
    drop(file);

    let people = get_persons();
    println!("\n{}", "I will test writing some json.".magenta());
    path.set_extension("json");
    println!("The path is now '{}'", path.display().to_string().yellow());
    let mut file = file::create(&path, Some(FileOpenOptions::Default))?;
    file.write_json(&people, Some(true))?;
    drop(file);

    println!("\n{}", "I will open the file and read it.".magenta());
    let file = file::open(&path)?;

    for line in file.read()? {
        println!("{}", line);
    }

    drop(file);

    println!("\n{}", "I will test writing some csv.".magenta());
    path.set_extension("csv");
    println!("The path is now '{}'", path.display().to_string().yellow());
    let mut file = file::create(&path, Some(FileOpenOptions::Default))?;
    let mut writer = file.create_delimited_writer(None, Some(true));

    for person in &people {
        writer.serialize(person)?;
    }

    drop(writer);

    println!("\n{}", "I will open the file and read it.".magenta());
    let mut file = file::open(&path)?;
    let mut reader = file.create_delimited_reader(None, Some(true));

    let headers = reader.headers()?;
    println!("{:?}", headers);

    for result in reader.deserialize() {
        let record: Person = result?;
        println!("{:?}", record);
    }

    drop(file);

    println!("\n{}", "I will test writing some tsv.".magenta());
    path.set_extension("tsv");
    println!("The path is now '{}'", path.display().to_string().yellow());
    let mut file = file::create(&path, Some(FileOpenOptions::Default))?;
    let mut writer = file.create_delimited_writer(Some(b'\t'), None);

    for person in &people {
        writer.serialize(person)?;
    }

    drop(writer);

    println!("\n{}", "I will open the file and read it.".magenta());
    let mut file = file::open(&path)?;
    let reader = file.create_delimited_reader(Some(b'\t'), None);

    for result in reader.into_records() {
        let record = result?;
        println!("{:?}", record);
    }

    drop(file);

    println!("\n{}", "I will delete the directory.".magenta());
    let path = path::take(&path, original_path_len);
    println!("The path is now '{}'", path.display().to_string().yellow());
    match directory::delete(&path) {
        Ok(_) => println!("{}", "Folder deleted.".green()),
        Err(e) => println!("Error: {}", e.to_string().red()),
    }
    Ok(())
}

fn test_url_func() -> Result<(), Box<dyn Error>> {
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

async fn test_reqwest_func() -> Result<(), Box<dyn Error>> {
    println!("\n{}", "Testing reqwest functions...".magenta());

    let baseUrl = url::create("https://dummy.restapiexample.com/api/v1")?;
    println!("baseUrl: {}", baseUrl);

    let url = baseUrl.join("/employees")?;
    println!(
        "I will try to get the employees list from the endpoint '{}'.",
        url
    );

    // get the employees list as json
    let mut response = reqwest::get(url).await?;
    let body = response.text().await?;

    println!("{:#?}", employees);

    Ok(())
}

fn print_batch(batch: u32, lines: Vec<String>) -> bool {
    println!("\nReading batch {}", batch);

    for line in lines {
        print!("{}", line);
    }

    true
}

fn get_persons() -> Vec<Person> {
    vec![
        Person {
            name: "Alice".to_string(),
            age: 25,
            email: Some("alice@example.com".to_string()),
            phones: Some(vec!["12345678".to_string()]),
        },
        Person {
            name: "Bob".to_string(),
            age: 30,
            email: Some("bob@example.com".to_string()),
            phones: Some(vec!["12345678".to_string(), "87654321".to_string()]),
        },
        Person {
            name: "Charlie".to_string(),
            age: 35,
            email: Some("charlie@example.com".to_string()),
            phones: None,
        },
    ]
}
