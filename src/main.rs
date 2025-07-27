use dialoguer::{Input, Select};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ask the user for the mods folder path
    let mods_path_str: String = Input::new()
        .with_prompt("Enter the full path to your Minecraft 'mods' folder")
        .interact_text()?;

    let mods_path = Path::new(&mods_path_str);

    if !mods_path.exists() || !mods_path.is_dir() {
        eprintln!("The specified path is not a valid folder: {:?}", mods_path);
        return Ok(());
    }

    // List .jar files in the folder
    let entries: Vec<PathBuf> = fs::read_dir(&mods_path)?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension().map_or(false, |ext| ext == "jar") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    if entries.is_empty() {
        println!("No .jar mod files found in {:?}", mods_path);
        return Ok(());
    }

    println!("Select a mod file to replace:");
    let options: Vec<String> = entries
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();

    let selection = Select::new().items(&options).default(0).interact()?;
    let target_file = &entries[selection];
    let original_size = fs::metadata(target_file)?.len();

    let replacement_path: PathBuf = Input::<String>::new()
        .with_prompt("Enter path to replacement file")
        .interact_text()?
        .into();

    if !replacement_path.exists() {
        eprintln!("Replacement file does not exist.");
        return Ok(());
    }

    let mut replacement_data = Vec::new();
    File::open(&replacement_path)?.read_to_end(&mut replacement_data)?;
    let replacement_size = replacement_data.len() as u64;

    if replacement_size > original_size {
        eprintln!(
            "Replacement file is larger ({} bytes) than original ({} bytes). Aborting.",
            replacement_size, original_size
        );
        return Ok(());
    }

    let padding_needed = original_size - replacement_size;
    replacement_data.extend(std::iter::repeat(0).take(padding_needed as usize));

    let mut file = File::create(target_file)?;
    file.write_all(&replacement_data)?;
    file.flush()?;

    println!(
        "Replaced '{}' with '{}'. Size: {} → {} (padded).",
        options[selection],
        replacement_path.display(),
        replacement_size,
        original_size
    );

    Ok(())
}
