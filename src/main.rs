use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use dialoguer::{Input, Select};
use rfd::FileDialog;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Select the replacement file...");
    let replacement_path = FileDialog::new()
        .set_title("Select Replacement .jar File")
        .add_filter("Jar Files", &["jar"])
        .pick_file()
        .ok_or("No file selected")?;

    // Validate extension
    if replacement_path.extension().and_then(|s| s.to_str()) != Some("jar") {
        eprintln!("❌ The selected file is not a .jar file.");
        return Ok(());
    }

    let mut replacement_data = Vec::new();
    File::open(&replacement_path)?.read_to_end(&mut replacement_data)?;
    let replacement_size = replacement_data.len() as u64;
    println!(
        "Selected replacement file: {}\nSize: {} bytes\n",
        replacement_path.display(),
        replacement_size
    );

    let mods_path_str: String = Input::<String>::new()
        .with_prompt("Enter the full path to your Minecraft 'mods' folder")
        .interact_text()?;

    let mods_path = Path::new(&mods_path_str);

    if !mods_path.exists() || !mods_path.is_dir() {
        eprintln!("The specified path is not a valid folder: {:?}", mods_path);
        return Ok(());
    }

    let mut entries: Vec<(PathBuf, u64)> = fs::read_dir(&mods_path)?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension().map_or(false, |ext| ext == "jar") {
                let size = fs::metadata(&path).ok()?.len();
                if replacement_size <= size {
                    Some((path, size))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    entries.sort_by_key(|(_, size)| ((*size as i64 - replacement_size as i64).abs()));

    if entries.is_empty() {
        println!("No suitable .jar mod files found in {:?}", mods_path);
        return Ok(());
    }

    println!("Select a mod file to replace:");

    let options: Vec<String> = entries
        .iter()
        .map(|(p, size)| {
            let diff = (*size as i64 - replacement_size as i64).abs();
            format!(
                "{} | {} bytes | Δ {} bytes",
                p.file_name().unwrap().to_string_lossy(),
                size,
                diff
            )
        })
        .collect();

    let selection = Select::new().items(&options).default(0).interact()?;
    let (target_path, original_size) = &entries[selection];

    if replacement_size > *original_size {
        eprintln!(
            "Replacement file is larger ({} bytes) than selected mod ({} bytes). Aborting.",
            replacement_size, original_size
        );
        return Ok(());
    }

    let padding_needed = original_size - replacement_size;
    replacement_data.extend(std::iter::repeat(0).take(padding_needed as usize));

    let mut file = File::create(target_path)?;
    file.write_all(&replacement_data)?;
    file.flush()?;

    println!(
        "Replaced '{}' with '{}'. Padded from {} → {} bytes.",
        target_path.file_name().unwrap().to_string_lossy(),
        replacement_path.display(),
        replacement_size,
        original_size
    );

    Ok(())
}
