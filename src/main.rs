use anyhow::{Context, Result};
use clap::Parser;
use clipboard::{ClipboardContext, ClipboardProvider};
use glob::Pattern;
use std::fs;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(author, version, about = "Copy directory contents to clipboard")]
struct Args {
    /// Recursively process subdirectories
    #[arg(short, long)]
    recursive: bool,

    /// Filter files by pattern (e.g., "*.rs")
    #[arg(short, long)]
    filter: Option<String>,
}

fn get_directory_listing(path: &str) -> Result<String> {
    let output = Command::new("ls")
        .arg("-l")
        .arg(path)
        .output()
        .context("Failed to execute ls command")?;
    
    String::from_utf8(output.stdout)
        .context("Failed to parse ls output")
}

fn read_file_contents<P: AsRef<Path>>(path: P) -> Result<String> {
    fs::read_to_string(path)
        .context("Failed to read file")
}

fn should_process_file(path: &Path, filter_pattern: Option<&Pattern>) -> bool {
    if let Some(pattern) = filter_pattern {
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            pattern.matches(file_name)
        } else {
            false
        }
    } else {
        true
    }
}

fn directory_has_matching_files(dir_path: &Path, filter_pattern: Option<&Pattern>) -> bool {
    WalkDir::new(dir_path)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .any(|entry| {
            entry.file_type().is_file() && should_process_file(entry.path(), filter_pattern)
        })
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Convert filter pattern if provided
    let filter_pattern = args.filter
        .as_ref()
        .map(|f| Pattern::new(f))
        .transpose()
        .context("Invalid filter pattern")?;
    
    // Initialize clipboard
    let mut ctx: ClipboardContext = ClipboardProvider::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize clipboard: {}", e))?;
    
    // Start building the output string
    let mut output = String::new();
    
    // Configure WalkDir based on recursive flag
    let mut walker = WalkDir::new(".")
        .min_depth(1);
    
    if !args.recursive {
        walker = walker.max_depth(1);
    }
    
    // Keep track of current directory to avoid duplicate listings
    let mut current_dir: Option<String> = None;
    
    // Process all entries
    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        
        if entry.file_type().is_file() && should_process_file(path, filter_pattern.as_ref()) {
            // If we're in a new directory that contains matching files, add its listing
            let dir_path = path.parent().unwrap().to_string_lossy().to_string();
            if current_dir.as_ref() != Some(&dir_path) {
                // For recursive mode, check if directory has matching files
                if !args.recursive || directory_has_matching_files(Path::new(&dir_path), filter_pattern.as_ref()) {
                    output.push_str(&format!("\n=== Directory: {} ===\n", dir_path));
                    if let Ok(listing) = get_directory_listing(&dir_path) {
                        output.push_str(&listing);
                    }
                    current_dir = Some(dir_path);
                }
            }
            
            // Add file contents
            if let Ok(contents) = read_file_contents(path) {
                output.push_str(&format!("\n=== File: {} ===\n", path.display()));
                output.push_str(&contents);
                output.push_str("\n");
            }
        }
    }
    
    // Copy to clipboard
    ctx.set_contents(output)
        .map_err(|e| anyhow::anyhow!("Failed to set clipboard contents: {}", e))?;
    
    println!("Directory contents and file contents have been copied to clipboard!");
    
    // Print summary of what was processed
    if let Some(pattern) = &args.filter {
        println!("Filtered files using pattern: {}", pattern);
    }
    if args.recursive {
        println!("Processed subdirectories recursively (showing only directories with matching files)");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{File, create_dir};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_should_process_file_no_filter() {
        let path = Path::new("example.rs");
        assert!(should_process_file(path, None));
    }

    #[test]
    fn test_should_process_file_with_matching_filter() {
        let pattern = Pattern::new("*.rs").unwrap();
        let path = Path::new("main.rs");
        assert!(should_process_file(path, Some(&pattern)));
    }

    #[test]
    fn test_should_process_file_with_non_matching_filter() {
        let pattern = Pattern::new("*.rs").unwrap();
        let path = Path::new("main.txt");
        assert!(!should_process_file(path, Some(&pattern)));
    }

    #[test]
    fn test_directory_has_matching_files() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        File::create(&file_path).unwrap();

        let pattern = Pattern::new("*.rs").unwrap();
        assert!(directory_has_matching_files(dir.path(), Some(&pattern)));
    }

    #[test]
    fn test_directory_has_no_matching_files() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let pattern = Pattern::new("*.rs").unwrap();
        assert!(!directory_has_matching_files(dir.path(), Some(&pattern)));
    }

    #[test]
    fn test_read_file_contents() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("testfile.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();

        let contents = read_file_contents(&file_path).unwrap();
        assert_eq!(contents.trim(), "Hello, world!");
    }

    #[test]
    fn test_read_file_contents_nonexistent_file() {
        // Reading a non-existent file should return an error
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("does_not_exist.txt");
        let result = read_file_contents(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_file_contents_directory_instead_of_file() {
        // Reading a directory should return an error
        let dir = tempdir().unwrap();
        let result = read_file_contents(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_get_directory_listing_valid_path() {
        let dir = tempdir().unwrap();
        // Create a file so listing isn't empty
        let file_path = dir.path().join("testfile.txt");
        File::create(&file_path).unwrap();

        // We expect some output (depends on OS, so just check non-empty success)
        let listing = get_directory_listing(dir.path().to_str().unwrap());
        assert!(listing.is_ok());
        assert!(!listing.unwrap().is_empty());
    }

    #[test]
    fn test_directory_has_matching_files_subdir() {
        let dir = tempdir().unwrap();

        // create subdirectory
        let sub_path = dir.path().join("sub");
        create_dir(&sub_path).unwrap();

        // create file in subdirectory
        let file_path = sub_path.join("test.rs");
        File::create(&file_path).unwrap();

        // pattern
        let pattern = Pattern::new("*.rs").unwrap();

        // now check
        assert!(directory_has_matching_files(dir.path(), Some(&pattern)));
    }

    #[test]
    fn test_invalid_filter_pattern() {
        // An intentionally invalid pattern
        let pattern = Pattern::new("[abc");
        assert!(pattern.is_err());
    }
}
