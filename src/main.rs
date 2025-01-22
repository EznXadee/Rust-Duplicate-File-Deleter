use clap::{Arg, Command};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use std::io::{Write};

// Function to calculate the SHA-256 hash of a file
fn calculate_hash(file_path: &Path) -> io::Result<String> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    hasher.update(&buffer);
    Ok(format!("{:x}", hasher.finalize()))
}

// Function to scan a directory and return a map of file hashes
fn scan_directory(path: &str) -> io::Result<HashMap<String, Vec<PathBuf>>> {
    let mut file_hash_map: HashMap<String, Vec<PathBuf>> = HashMap::new();

    for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            let file_path = entry.path().to_path_buf();
            match calculate_hash(&file_path) {
                Ok(hash) => {
                    file_hash_map.entry(hash).or_insert(Vec::new()).push(file_path);
                }
                Err(_) => continue, // Skip files that fail to read
            }
        }
    }
    Ok(file_hash_map)
}

// Function to ask the user for confirmation before deleting files
fn ask_for_confirmation() -> bool {
    print!("Do you want to delete these files? (y/n): ");
    io::stdout().flush().unwrap(); // Ensure the prompt is printed immediately

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_lowercase() == "y"
}

#[tokio::main]
async fn main() {
    let matches = Command::new("Duplicate File Finder")
        .version("1.0")
        .about("Detects and deletes duplicate files on your PC")
        .arg(
            Arg::new("directory")
                .help("Directory to scan for duplicate files")
                .required(true)
                .index(1),
        )
        .get_matches();

    let directory = matches.get_one::<String>("directory").unwrap();

    match scan_directory(directory) {
        Ok(file_hash_map) => {
            let duplicates: Vec<Vec<PathBuf>> = file_hash_map
                .into_iter()
                .filter(|(_, files)| files.len() > 1)
                .map(|(_, files)| files)
                .collect();

            if duplicates.is_empty() {
                println!("No duplicates found.");
            } else {
                println!("Found duplicate files:");
                for group in duplicates {
                    println!("{:?}", group);
                    // Ask the user for confirmation before deleting the files
                    if ask_for_confirmation() {
                        let mut keep_files = vec![];
                        for (index, file) in group.iter().enumerate() {
                            if index != 0 {
                                // Skip the first file, keep it, and delete the others
                                if let Err(e) = fs::remove_file(file) {
                                    eprintln!("Failed to delete {:?}: {}", file, e);
                                } else {
                                    println!("Deleted: {:?}", file);
                                }
                            } else {
                                keep_files.push(file.clone());
                            }
                        }
                    } else {
                        println!("Skipped deleting duplicates in this group.");
                    }
                }
            }
        }
        Err(e) => eprintln!("Error scanning directory: {}", e),
    }
}
