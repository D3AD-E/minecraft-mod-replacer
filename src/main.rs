use dialoguer::{Input, Select};
use rfd::FileDialog;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Select the replacement file...");
    let replacement_path = FileDialog::new()
        .set_title("Select Replacement .jar File")
        .add_filter("Jar Files", &["jar"])
        .pick_file()
        .ok_or("No file selected")?;

    // Validate extension
    if replacement_path.extension().and_then(|s| s.to_str()) != Some("jar") {
        eprintln!("The selected file is not a .jar file.");
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

    if padding_needed > 0 {
        println!("Attempting to pad {} bytes...", padding_needed);

        if let Some(padded_data) = pad_zip_file(replacement_data.clone(), padding_needed as usize)?
        {
            let mut file = File::create(target_path)?;
            file.write_all(&padded_data)?;
            file.flush()?;
        } else {
            // Fallback: simple append with warning
            eprintln!("⚠️  Warning: Could not pad using ZIP comment. Using simple append method.");
            eprintln!(
                "This may cause issues with strict ZIP parsers, but often works in practice."
            );

            let mut padded_data = replacement_data;
            let padding = vec![0u8; padding_needed as usize];
            padded_data.extend(padding);

            let mut file = File::create(target_path)?;
            file.write_all(&padded_data)?;
            file.flush()?;
        }
    } else {
        // No padding needed, write directly
        let mut file = File::create(target_path)?;
        file.write_all(&replacement_data)?;
        file.flush()?;
    }

    println!(
        "Replaced '{}' with '{}'. Padded from {} → {} bytes.",
        target_path.file_name().unwrap().to_string_lossy(),
        replacement_path.display(),
        replacement_size,
        original_size
    );

    Ok(())
}

fn pad_zip_file(
    mut data: Vec<u8>,
    padding_size: usize,
) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
    // Maximum comment size in ZIP format is 65535 bytes
    if padding_size > 65535 {
        eprintln!("Cannot pad more than 65535 bytes using ZIP comment field");
        return Ok(None);
    }

    // Try multiple methods to find the EOCD
    if let Some(eocd_start) = find_eocd(&data) {
        let current_comment_len =
            u16::from_le_bytes([data[eocd_start + 20], data[eocd_start + 21]]) as usize;

        let new_comment_len = current_comment_len + padding_size;

        if new_comment_len > 65535 {
            eprintln!("Total comment length would exceed ZIP limit of 65535 bytes");
            return Ok(None);
        }
        let new_comment_len_bytes = (new_comment_len as u16).to_le_bytes();
        data[eocd_start + 20] = new_comment_len_bytes[0];
        data[eocd_start + 21] = new_comment_len_bytes[1];
        let padding = vec![b'#'; padding_size];
        data.extend(padding);

        Ok(Some(data))
    } else {
        Ok(None)
    }
}

fn find_eocd(data: &[u8]) -> Option<usize> {
    let eocd_signature = [0x50, 0x4b, 0x05, 0x06];

    let search_start = data.len().saturating_sub(65557);
    for i in (search_start..data.len().saturating_sub(3)).rev() {
        if i + 22 <= data.len() && data[i..i + 4] == eocd_signature {
            let comment_len = u16::from_le_bytes([data[i + 20], data[i + 21]]) as usize;
            if i + 22 + comment_len <= data.len() {
                return Some(i);
            }
        }
    }
    println!("Standard EOCD search failed, trying thorough search...");
    for i in (0..data.len().saturating_sub(21)).rev() {
        if data[i..i + 4] == eocd_signature {
            if i + 22 <= data.len() {
                let comment_len = u16::from_le_bytes([data[i + 20], data[i + 21]]) as usize;
                if i + 22 + comment_len <= data.len() {
                    println!("Found EOCD at position {}", i);
                    return Some(i);
                }
            }
        }
    }

    println!(
        "Could not find valid EOCD record in file of {} bytes",
        data.len()
    );
    None
}
