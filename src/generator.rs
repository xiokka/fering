use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::io::prelude::*;

const REDIRECT_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta http-equiv="refresh" content="0; URL='$URL'" />
    <title>Redirecting...</title>
</head>
<body>
    <p>Redirecting...</p>
    <p>$URL</p>
</body>
</html>
"#;


// Read file content into a String
pub fn read_file_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    fs::read_to_string(path)
}

// Replace placeholders in HTML content
pub fn replace_placeholders(html_content: &str, placeholders: &HashMap<String, String>) -> String {
    let mut result = html_content.to_string();
    for (placeholder, replacement) in placeholders {
        result = result.replace(placeholder, replacement);
    }
    result
}

// Write HTML content to a file
pub fn write_html_file<P: AsRef<Path>>(path: P, content: &str) -> io::Result<()> {
    fs::write(path, content.as_bytes())
}

// Create directories if they don't exist
pub fn create_directories<P: AsRef<Path>>(path: P) -> io::Result<()> {
    if !path.as_ref().exists() {
        fs::create_dir_all(path)
    } else {
        Ok(())
    }
}

// Convert text to HTML
pub fn txt_to_html(content: Vec<u8>) -> Vec<u8> {
    let text = String::from_utf8_lossy(&content);
    let html_content = format!("<p>{}</p>", text.replace("\n", "</p><p>"));
    html_content.into_bytes()
}

// Get unique tags from a tags file
pub fn get_tags(tags_file_path: &str) -> HashSet<String> {
    let content = match fs::read_to_string(tags_file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading tags file {}: {}", tags_file_path, e);
            return HashSet::new();
        }
    };

    let mut unique_tags = HashSet::new();
    for tag in content.split_whitespace() {
        let trimmed = tag.trim();
        if !trimmed.is_empty() {
            unique_tags.insert(trimmed.to_string());
        }
    }

    unique_tags
}

// Filter users by tags
pub fn filter_users_by_tag() -> HashMap<String, Vec<PathBuf>> {
    let users_dir = Path::new("users");
    let mut tags_map = HashMap::new();

    if !users_dir.exists() || !users_dir.is_dir() {
        eprintln!("Users directory does not exist or is not a directory.");
        return tags_map;
    }

    if let Ok(users) = fs::read_dir(users_dir) {
        for user in users.filter_map(Result::ok) {
            let path = user.path();
            if !path.is_dir() {
                continue;
            }
            let tags_file_path = path.join("tags.txt");
            let tags = get_tags(tags_file_path.to_str().unwrap_or(""));

            for tag in tags {
                tags_map.entry(tag)
                    .or_insert_with(Vec::new)
                    .push(path.clone());
            }
        }
    } else {
        eprintln!("Failed to read the users directory.");
    }

    tags_map
}

// Copy directory and its contents recursively
pub fn copy_directory<P: AsRef<Path>>(source: P, destination: P) -> io::Result<()> {
    let source = source.as_ref();
    let destination = destination.as_ref();

    if !source.is_dir() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Source is not a directory"));
    }

    if !destination.exists() {
        fs::create_dir_all(destination)?;
    }

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let relative_path = path.strip_prefix(source).unwrap(); // Path relative to source

        let new_destination = destination.join(relative_path);

        if path.is_dir() {
            copy_directory(path, new_destination)?;
        } else {
            fs::copy(path, new_destination)?;
        }
    }

    Ok(())
}

// Generate pages for users
pub fn generate_user_pages(base_html: &str, users_dir: &Path, public_users_dir: &Path) -> io::Result<()> {
    // Collect user directories and their filenames
    let mut user_dirs: Vec<(String, PathBuf)> = fs::read_dir(users_dir)?
        .filter_map(Result::ok)
        .map(|entry| {
            let path = entry.path();
            let filename = path.file_name().unwrap().to_string_lossy().to_string();
            (filename, path)
        })
        .collect();

    // Sort directories by filenames
    user_dirs.sort_by_key(|(filename, _)| filename.clone());

    let num_users = user_dirs.len();

    for (index, (_, user_path)) in user_dirs.iter().enumerate() {
        let username = user_path.file_name().unwrap().to_str().unwrap_or("Unknown");
        let new_user_dir = public_users_dir.join(username);
        create_directories(&new_user_dir)?;

        // Paths to user files
        let url_file_path = user_path.join("url.txt");
        let description_file_path = user_path.join("description.html");

        // Read URL content
        let url_content = match fs::read_to_string(&url_file_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading url.txt for {}: {}", username, e);
                continue;
            }
        };

        // Determine previous and next user URLs with circular references
        let prev_user = if index == 0 {
            &user_dirs[num_users - 1].1 // Last user
        } else {
            &user_dirs[index - 1].1
        };
        let next_user = if index == num_users - 1 {
            &user_dirs[0].1 // First user
        } else {
            &user_dirs[index + 1].1
        };

        let prev_url = match fs::read_to_string(prev_user.join("url.txt")) {
            Ok(content) => content,
            Err(_) => "Previous user URL not available".to_string(),
        };

        let next_url = match fs::read_to_string(next_user.join("url.txt")) {
            Ok(content) => content,
            Err(_) => "Next user URL not available".to_string(),
        };

        // Read description and convert to HTML
        let description_html = if description_file_path.exists() {
            fs::read(&description_file_path)?
        } else {
            eprintln!("No description.html found in {:?}", user_path);
            continue;
        };

        // Prepare placeholders for HTML replacement
        let mut placeholders = HashMap::new();
	placeholders.insert("$URL".to_string(), format!("<a href=\"{}\">Website</a>", url_content.clone()));
        placeholders.insert("$PREV_URL".to_string(), "".to_string() );
        placeholders.insert("$NEXT_URL".to_string(), "".to_string() );
        placeholders.insert("$CONTENT".to_string(), String::from_utf8_lossy(&description_html).to_string());
        placeholders.insert("$TITLE".to_string(), username.to_string());
        placeholders.insert("$NAVCLOUD".to_string(), "".to_string());

        // Generate final HTML content
        let final_html_content = replace_placeholders(base_html, &placeholders);

        // Write HTML file to user's directory
        write_html_file(new_user_dir.join("index.html"), &final_html_content)?;

        // Generate and write redirect HTML for previous and next users
        let prev_placeholders = {
            let mut hm = HashMap::new();
            hm.insert("$URL".to_string(), prev_url.clone());
            hm
        };
        let next_placeholders = {
            let mut hm = HashMap::new();
            hm.insert("$URL".to_string(), next_url.clone());
            hm
        };

        let previous_html_content = replace_placeholders(REDIRECT_TEMPLATE, &prev_placeholders);
        let next_html_content = replace_placeholders(REDIRECT_TEMPLATE, &next_placeholders);

        write_html_file(new_user_dir.join("previous.html"), &previous_html_content)?;
        write_html_file(new_user_dir.join("next.html"), &next_html_content)?;
    }

    Ok(())
}


// Generate tag pages
pub fn generate_tag_pages(base_html: &str, tags_map: &HashMap<String, Vec<PathBuf>>, public_dir: &Path) -> io::Result<()> {
    for (tag, paths) in tags_map {
        let tag_dir = public_dir.join(tag);
        create_directories(&tag_dir)?;

        let mut tag_content = String::new();
        for path in paths {
            let username = path.file_name().unwrap().to_str().unwrap_or("Unknown");
            let user_link = format!("<a href=\"../users/{}/index.html\">{}</a><br>", username, username);
            tag_content.push_str(&user_link);
        }

        let tag_html_content = replace_placeholders(
            &base_html,
            &[
                ("$CONTENT".to_string(), tag_content),
                ("$TITLE".to_string(), tag.to_string()),
                ("$NAVCLOUD".to_string(), "".to_string()),
            	("$URL".to_string(), "".to_string() ),
            	("$PREV_URL".to_string(), "".to_string() ),
            	("$NEXT_URL".to_string(), "".to_string() ),
            ].iter().cloned().collect()
        );
        write_html_file(tag_dir.join("index.html"), &tag_html_content)?;
    }
    Ok(())
}

// Generate the site
pub fn generate_site() -> io::Result<()> {
    let public_dir = Path::new("public");
    let users_dir = public_dir.join("users");
    let root_users_dir = Path::new("users");
    let base_html_path = Path::new("static").join("base.html");
    let about_txt_path = Path::new("static").join("about.html");
    let projectname_path = Path::new("projectname.txt");

    // Read base HTML
    let base_html = match read_file_to_string(&base_html_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read base HTML file: {}", e);
            return Err(e);
        }
    };

    // Read other static content
    let about_txt_content = match read_file_to_string(&about_txt_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read about.html: {}", e);
            return Err(e);
        }
    };

    let project_name = match read_file_to_string(&projectname_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read projectname.txt: {}", e);
            return Err(e);
        }
    };

    // Create public directories
    create_directories(&public_dir)?;
    create_directories(&users_dir)?;

    // Copy static files
    let static_source = Path::new("static");
    let static_dest = public_dir.join("static");
    copy_directory(static_source, &static_dest)?;

    // Copy images files
    let images_source = Path::new("images");
    let images_dest = public_dir.join("images");
    copy_directory(images_source, &images_dest)?;

    // Generate user pages
    generate_user_pages(&base_html, &root_users_dir, &users_dir)?;

    // Generate tag pages
    let tags_map = filter_users_by_tag();
    generate_tag_pages(&base_html, &tags_map, &public_dir)?;

    // Create navigation cloud. Contains links to each tag index
    let nav_cloud = tags_map.keys()
        .map(|tag| format!("<a href=\"{}/index.html\">{}</a>", tag, tag))
        .collect::<String>();

    // Replace the $NAVCLOUD placeholder in about_txt_content with tags
    let parsed_about_txt_content = replace_placeholders(
        &about_txt_content,
        &[
            ("$NAVCLOUD".to_string(), nav_cloud),
        ].iter().cloned().collect()
    );

    // Generate the root index.html
    let root_index_html_content = replace_placeholders(
        &base_html,
        &[
            ("$CONTENT".to_string(), parsed_about_txt_content),
            ("$TITLE".to_string(), project_name),
            ("$NAVCLOUD".to_string(), "".to_string()),
            ("$URL".to_string(), "".to_string() ),
            ("$PREV_URL".to_string(), "".to_string() ),
            ("$NEXT_URL".to_string(), "".to_string() ),
        ].iter().cloned().collect()
    );
    write_html_file(public_dir.join("index.html"), &root_index_html_content)?;

    // Generate users index.html
    let mut users_index_content = String::new();
    for user in fs::read_dir(users_dir.clone())? {
        let user = user?;
        let user_path = user.path();
        if !user_path.is_dir() {
            continue;
        }
        let username = user_path.file_name().unwrap().to_str().unwrap_or("Unknown");
        let user_link = format!("<a href=\"{}/index.html\">{}</a><br>", username, username);
        users_index_content.push_str(&user_link);
    }

    let users_index_html_content = replace_placeholders(
        &base_html,
        &[
            ("$CONTENT".to_string(), users_index_content),
            ("$TITLE".to_string(), "Users".to_string()),
            ("$NAVCLOUD".to_string(), "".to_string()),
            ("$URL".to_string(), "".to_string() ),
            ("$PREV_URL".to_string(), "".to_string() ),
            ("$NEXT_URL".to_string(), "".to_string() ),
        ].iter().cloned().collect()
    );
    write_html_file(users_dir.join("index.html"), &users_index_html_content)?;

    Ok(())
}
