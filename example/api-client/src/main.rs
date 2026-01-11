//! HTTP client example for shopkeep
//!
//! Demonstrates how to interact with the shopkeep extension registry API.
//!
//! Usage:
//!   cargo run -p api-client [--url http://localhost:8080]

use shopkeep_core::{Details, Page, Summary, Version};

const DEFAULT_URL: &str = "http://localhost:8080";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = std::env::args()
        .nth(1)
        .filter(|arg| !arg.starts_with('-'))
        .or_else(|| {
            std::env::args()
                .skip_while(|arg| arg != "--url")
                .nth(1)
        })
        .unwrap_or_else(|| DEFAULT_URL.to_string());

    println!("Shopkeep API Client Demo");
    println!("========================\n");
    println!("Connecting to: {}\n", base_url);

    let client = reqwest::Client::new();

    // 1. List all extensions
    println!("1. Listing extensions...\n");
    let page: Page<Summary> = client
        .get(format!("{}/api/v1/extensions", base_url))
        .send()
        .await?
        .json()
        .await?;

    println!(
        "   Found {} extension(s) (page {}/{}):\n",
        page.total, page.page, page.total_pages
    );

    for ext in &page.items {
        println!("   - {} v{}", ext.id, ext.version);
        println!("     {}", ext.description);
        println!("     Author: {}, License: {}", ext.author, ext.license);
        println!();
    }

    // Pick the first extension for further demo
    let Some(first) = page.items.first() else {
        println!("No extensions found in registry.");
        return Ok(());
    };

    let ext_id = &first.id;

    // 2. Get extension details
    println!("2. Getting details for '{}'...\n", ext_id);
    let details: Details = client
        .get(format!("{}/api/v1/extensions/{}", base_url, ext_id))
        .send()
        .await?
        .json()
        .await?;

    println!("   Name: {}", details.name);
    println!("   Version: {}", details.version);
    println!("   Description: {}", details.description);
    println!("   Author: {}", details.author);
    println!("   License: {}", details.license);
    if !details.keywords.is_empty() {
        println!("   Keywords: {}", details.keywords.join(", "));
    }
    if !details.capabilities.is_empty() {
        println!("   Capabilities: {}", details.capabilities.join(", "));
    }
    if !details.operations.is_empty() {
        println!("   Operations: {}", details.operations.join(", "));
    }
    if !details.versions.is_empty() {
        println!("   Available versions: {}", details.versions.join(", "));
    }
    println!();

    // 3. List versions
    println!("3. Listing versions for '{}'...\n", ext_id);
    let versions: Vec<Version> = client
        .get(format!("{}/api/v1/extensions/{}/versions", base_url, ext_id))
        .send()
        .await?
        .json()
        .await?;

    for v in &versions {
        println!("   - v{}", v.version);
        println!("     Size: {} bytes", v.size_bytes);
        println!("     SHA256: {}", v.checksum_sha256);
        println!("     Created: {}", v.created_at);
        println!();
    }

    // 4. Download the latest version
    println!("4. Downloading latest version of '{}'...\n", ext_id);
    let response = client
        .get(format!(
            "{}/api/v1/extensions/{}/latest/download",
            base_url, ext_id
        ))
        .send()
        .await?;

    let content_disposition = response
        .headers()
        .get("content-disposition")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let bytes = response.bytes().await?;
    println!("   Downloaded {} bytes", bytes.len());
    println!("   Content-Disposition: {}", content_disposition);

    println!("\nDemo complete!");

    Ok(())
}
