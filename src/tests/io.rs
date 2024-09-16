use rand::{distributions::Alphanumeric, Rng};
use rustmix::{
    io::{
        directory,
        file::{self, FileEx},
        path::{self, IntoPath, PathEx},
    },
    Result,
};
use std::{
    io::{stdin, LineWriter, Write},
    path::PathBuf,
};

use super::*;

pub fn test_path() -> Result<()> {
    println!("\nTesting path functions...");
    let curdir = directory::current();
    let path = (curdir.as_str(), "MyFile.txt").into_path();
    println!("{}", path.display());

    let path = (curdir.as_str(), "My Folder", "MyFile.txt").into_path();
    println!("{}", path.display());

    let path = (curdir.as_str(), "My Folder", "My Subfolder", "MyFile.txt").into_path();
    println!("{}", path.display());

    let path: PathBuf = [curdir.as_str(), "My Folder", "My Subfolder", "", "NonEmpty"].into_path();
    println!("{}", path.display());

    let path = "./files/audio";
    println!("\nI will find SOME files in '{}'.", &path);

    for entry in path::lst_match(format!("{}/*.mp3", &path)).unwrap() {
        println!("{}", entry.display());
    }

    println!(
        "\nI will find some files in '{}' that start with f and end with n.",
        &path
    );

    for entry in path::lst_match_filtered(format!("{}/*.mp3", &path), |e| {
        let fname = e.file_stem().unwrap().to_str().unwrap();
        fname.starts_with('f') && fname.ends_with('n')
    })
    .unwrap()
    {
        println!("{}", entry.display());
    }

    println!("\nI will create a temp folder to test a few things.");
    let tmpdir = curdir.join("tmp");
    directory::create(&tmpdir)?;
    println!("Temp folder created.");

    println!(
        "I will copy ALL files from './files/audio' to '{}'.",
        &tmpdir.display()
    );
    path::cpy("./files/audio/*.*", &tmpdir)?;
    println!("I will print ALL the new files.");

    for entry in path::lst(&tmpdir).unwrap() {
        println!("{}", entry.display());
    }

    let new_folder = tmpdir.join("new_folder");
    println!(
        "\nI will move SOME files to a new folder '{}'.",
        new_folder.display()
    );
    path::mov(&format!("{}/*.wav", tmpdir.display()), &new_folder)?;

    for entry in path::lst(&new_folder).unwrap() {
        println!("{}", entry.display());
    }

    println!("\nI will rename the new folder.");
    path::ren(&new_folder, "new_folder_renamed")?;

    println!("\nI will delete the temp folder.");
    path::del(&tmpdir)?;

    Ok(())
}

pub fn test_directory() -> Result<()> {
    println!("\nTesting directory functions...");

    let curdir = directory::current();
    let original_path_len = curdir.components().count();
    // could use that or simply curdir.join()
    let path = (curdir.as_str(), "My Folder", "My Subfolder", "NonEmpty").into_path();

    println!(
        "Current Dir: '{}'. It has {} components.",
        &curdir.display(),
        &original_path_len
    );

    println!("Target Dir: '{}'", &path.display());

    let exists = directory::exists(&path);
    println!(
        "Does the directory exist? {}",
        if exists { "Yes" } else { "No" }
    );

    if !exists {
        println!("\nI will try to create the directory '{}'", &path.display());

        let created_err = match directory::create(&path) {
            Ok(_) => None,
            Err(e) => Some(format!("Error: {}", e)),
        };

        if created_err.is_none() {
            println!("Directory created using create('{}')", &path.display());
        } else {
            println!("{}", created_err.unwrap());
            directory::ensure(&path)?;
            println!("Directory created using ensure('{}')", &path.display());
        }
    }

    let parts = path::split(&path.as_str());

    for part in parts {
        println!("{}", &part);
    }

    println!("\nI will delete the directory.");
    let path = path.take(original_path_len + 1);
    println!("The path is now '{}'", &path.display());
    delete_dir(&path)?;

    Ok(())
}

pub fn test_file() -> Result<()> {
    println!("\nTesting file functions...");

    let curdir = directory::current();
    let original_path_len = curdir.components().count();
    let mut path = curdir.join("My Folder/My Subfolder/NonEmpty");

    println!(
        "\n\nCurrent Dir: '{}'. It has {} components.",
        &curdir.display(),
        &original_path_len
    );

    let exists = path.exists();
    println!("Does the file exist? {}", if exists { "Yes" } else { "No" });

    println!("I will create or open the file '{}'", &path.display());
    let file = file::create(&path)?;
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
    let mut file = file::create(&path)?;
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
    let mut file = file::create(&path)?;
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
    let mut file = file::create(&path)?;
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
    let path = path.take(original_path_len + 1);
    println!("The path is now '{}'", &path.display());
    delete_dir(&path)?;

    Ok(())
}

fn delete_dir(path: &PathBuf) -> Result<()> {
    print!("Do you want to delete the directory? (y/n): ");
    std::io::stdout().flush()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() == "y" {
        match path::del(&path) {
            Ok(_) => println!("Folder deleted."),
            Err(e) => println!("Error: {}", e),
        }
    } else {
        println!("Directory not deleted.");
    }

    Ok(())
}
