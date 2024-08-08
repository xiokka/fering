mod generator;
pub use generator::*;

use std::collections::HashSet;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Write;

// Basic HTML file structure, for initial project generation
const HTML_CONTENT: &str = include_str!("base.html");

// Basic CSS
const CSS_CONTENT: &str = include_str!("style.css");

// Subdirectories created when initializing a new project
const PROJECT_SUBDIRECTORIES: &[&str] = &["users", "images", "static"];

fn create_directories(root: &str, sub_dirs: &[&str]) -> io::Result<()> {
    let root_path = PathBuf::from(root);
    for dir in sub_dirs {
        let mut path_buf = root_path.join(dir);
        fs::create_dir_all(&path_buf)?;
        println!("Created directory: {}", path_buf.display());
    }
    Ok(())
}

fn initialize_project(project_name: &str) -> io::Result<()> {
    create_directories(project_name, &PROJECT_SUBDIRECTORIES)?;

    let projectname_file_path = Path::new(project_name).join("projectname.txt");
    let mut projectname_file = File::create(projectname_file_path)?;
    projectname_file.write_all(project_name.as_bytes())?;

    let html_path = Path::new(project_name).join("static").join("base.html");
    let mut base_html = File::create(html_path)?;
    base_html.write_all(HTML_CONTENT.as_bytes())?;

    let css_path = Path::new(project_name).join("static").join("style.css");
    let mut base_css = File::create(css_path)?;
    base_css.write_all(CSS_CONTENT.as_bytes())?;

    let about_file_path = Path::new(project_name).join("static").join("about.html");
    let mut about_file = File::create(about_file_path)?;
    about_file.write_all("<p>This will be shown at the blog index. Edit me at static/about.html</p> $NAVCLOUD".as_bytes())?;

    println!("HTML content saved to static/base.html");
    println!("Project name saved to projectname.txt");

    Ok(())
}

fn initialize_user(user_name: &str, user_url:&str) -> io::Result<()> {
    let users_dir = Path::new("users");

    if !users_dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "The 'users' directory does not exist.",
        ));
    }

    let user_path = users_dir.join(user_name);
    fs::create_dir_all(&user_path)?;
    println!("Created user directory: {}", user_path.display());

    let url_file_path = user_path.join("url.txt");
    let description_file_path = user_path.join("description.html");
    let tags_file_path = user_path.join("tags.txt");

    let mut url = fs::File::create(url_file_path)?;
    fs::File::create(description_file_path)?;
    fs::File::create(tags_file_path)?;
    url.write_all(user_url.as_bytes())?;
    println!("Created files: url.txt, description.html, and tags.txt");

    Ok(())
}

fn project_exists() -> bool {
    let path = Path::new("projectname.txt");
    path.exists()
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        println!("No arguments provided.");
        return Ok(());
    }

    let command = &args[1];
    match command.as_str() {
        "new_webring" => {
            if args.len() < 3 {
                eprintln!("Error: No webring name provided.");
                return Ok(());
            }
            let project_name = &args[2];
            initialize_project(project_name)?;
        }

        "add_user" => {
            if !project_exists() {
                eprintln!("Error: No webring found. Please run from webring project root directory.");
                return Ok(());
            }
            if args.len() < 4 {
                eprintln!("Error: No user name or URL provided.");
                return Ok(());
            }
            let user_name = &args[2];
	    let user_url = &args[3];
            initialize_user(user_name, user_url)?;
        }

        "print_users_by_tag" => {
            if !project_exists() {
                eprintln!("Error: No webring found. Please run from webring project root directory.");
                return Ok(());
            }
            let tags_map = filter_users_by_tag();

            for (tag, paths) in tags_map {
                println!("Tag: {}", tag);
                for path in paths {
                    println!("  Path: {}", path.display());
                }
            }
        }

        "generate" => {
            if !project_exists() {
                eprintln!("Error: No webring found. Please run from webring project root directory.");
                return Ok(());
            }
            if let Err(e) = generate_site() {
                eprintln!("Error generating site: {}", e);
            }
        }

        _ => println!("Unrecognized command: {}", command),
    }

    Ok(())
}
