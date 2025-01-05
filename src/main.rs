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