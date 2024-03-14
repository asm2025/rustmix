use rand::{distributions::Alphanumeric, Rng};
use std::{
    error::Error,
    io::{LineWriter, Write},
    path::PathBuf,
};

use rustmix::io::{
    directory,
    file::{self, FileEx, FileOpenOptions},
    path,
};

use super::{get_employees, print_batch, Employee};

pub fn test_path_func() -> Result<(), Box<dyn Error>> {
    println!("\nTesting path functions...");

    let curdir = directory::current().into_os_string().into_string().unwrap();

    let path = path::from(&curdir).join("MyFile.txt");
    println!("{}", path.display());

    let path = path::from(&curdir).join("My Folder").join("MyFile.txt");
    println!("{}", path.display());

    let path = path::from(&curdir)
        .join("My Folder")
        .join("My Subfolder")
        .join("MyFile.txt");
    println!("{}", path.display());

    let path: PathBuf = [&curdir, "My Folder", "My Subfolder", "", "NonEmpty"]
        .iter()
        .collect();
    println!("{}", path.display());
    Ok(())
}

pub fn test_directory_func() -> Result<(), Box<dyn Error>> {
    println!("\nTesting directory functions...");

    let curdir = directory::current();
    let original_path_len = curdir.components().count();
    let path = path::from(curdir.to_str().unwrap())
        .join("My Folder")
        .join("My Subfolder")
        .join("NonEmpty");

    println!(
        "Current Dir: '{}'. It has {} components.",
        curdir.display().to_string(),
        original_path_len.to_string()
    );

    let exists = directory::exists(&path);
    println!(
        "Does the directory exist? {}",
        if exists { "Yes" } else { "No" }
    );

    if !exists {
        println!(
            "\nI will try to create the directory '{}'",
            path.display().to_string()
        );

        let created = match directory::create(&path) {
            Ok(_) => None,
            Err(e) => Some(format!("Error: {}", e)),
        };

        if created.is_none() {
            println!("Directory created using create()");
        } else {
            println!("{}", created.unwrap());
        }
    }

    println!("\n{}", "I will delete the directory.");
    let path = path::take(&path, original_path_len);
    println!("The path is now '{}'", path.display());
    match directory::delete(&path) {
        Ok(_) => println!("Directory deleted."),
        Err(e) => println!("Error: {}", e),
    }
    Ok(())
}

pub fn test_file_func() -> Result<(), Box<dyn Error>> {
    println!("\nTesting file functions...");

    let curdir = directory::current();
    let original_path_len = curdir.components().count();
    let mut path = path::from(curdir.to_str().unwrap())
        .join("My Folder")
        .join("My Subfolder")
        .join("NonEmpty");

    println!(
        "\n\nCurrent Dir: '{}'. It has {} components.",
        curdir.display(),
        original_path_len
    );

    let exists = path::exists(&path);
    println!("Does the file exist? {}", if exists { "Yes" } else { "No" });

    println!("I will create or open the file '{}'", &path.display());
    let file = file::create(&path, Some(FileOpenOptions::Default))?;
    println!("File created or openned.");

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
        "\nI will try to open the file '{}' and read it.",
        &path.display()
    );

    let file = file::open(&path)?;

    for line in file.read()? {
        println!("{}", line);
    }

    drop(file);

    println!("\nI will apply the filter now.");

    let file = file::open(&path)?;

    for line in file.read_filtered(|e: &str| !e.contains("12345"))? {
        println!("{}", line);
    }

    drop(file);

    println!("\nI will open the file and read it in batches of 5 lines.");
    let file = file::open(&path)?;
    file.read_batch(5, |batch, lines| print_batch(batch, lines))?;
    drop(file);

    println!("\nI will open the file and read it in batches of 5 lines and apply the filter.");
    let file = file::open(&path)?;
    file.read_batch_filtered(
        5,
        |e: &str| !e.contains("12345"),
        |batch, lines| print_batch(batch, lines),
    )?;
    drop(file);

    let employees = get_employees(3);
    println!("\nI will test writing some json.");
    path.set_extension("json");
    println!("The path is now '{}'", path.display());
    let mut file = file::create(&path, Some(FileOpenOptions::Default))?;
    file.write_json(&employees, Some(true))?;
    drop(file);

    println!("\nI will open the file and read it.");
    let file = file::open(&path)?;

    for line in file.read()? {
        println!("{}", line);
    }

    drop(file);

    println!("\nI will test writing some csv.");
    path.set_extension("csv");
    println!("The path is now '{}'", path.display());
    let mut file = file::create(&path, Some(FileOpenOptions::Default))?;
    let mut writer = file.create_delimited_writer(None, Some(true));

    for employee in &employees {
        writer.serialize(employee)?;
    }

    drop(writer);

    println!("\nI will open the file and read it.");
    let mut file = file::open(&path)?;
    let mut reader = file.create_delimited_reader(None, Some(true));

    let headers = reader.headers()?;
    println!("{:?}", headers);

    for result in reader.deserialize() {
        let record: Employee = result?;
        println!("{:?}", record);
    }

    drop(file);

    println!("\nI will test writing some tsv.");
    path.set_extension("tsv");
    println!("The path is now '{}'.", path.display());
    let mut file = file::create(&path, Some(FileOpenOptions::Default))?;
    let mut writer = file.create_delimited_writer(Some(b'\t'), None);

    for person in &employees {
        writer.serialize(person)?;
    }

    drop(writer);

    println!("\nI will open the file and read it.");
    let mut file = file::open(&path)?;
    let reader = file.create_delimited_reader(Some(b'\t'), None);

    for result in reader.into_records() {
        let record = result?;
        println!("{:?}", record);
    }

    drop(file);

    println!("\nI will delete the directory.");
    let path = path::take(&path, original_path_len);
    println!("The path is now '{}'", path.display());
    match directory::delete(&path) {
        Ok(_) => println!("Folder deleted."),
        Err(e) => println!("Error: {}", e),
    }
    Ok(())
}
