use clap::Parser;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufRead, BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Semaphore};

const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// URLProber - Lightning-fast concurrent URL prober with intelligent retry logic
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input file containing URLs (one per line). Use '-' for stdin
    #[arg(short = 'f', long)]
    urls_file: Option<PathBuf>,

    /// Output JSON file path
    #[arg(short = 'o', long)]
    output: PathBuf,

    /// (Optional) Maximum concurrent requests
    #[arg(long, default_value_t = 20)]
    concurrency: usize,

    /// (Optional) Number of retry attempts for failed requests
    #[arg(long, default_value_t = 2)]
    retries: usize,

    /// (Optional) Delay between retries in milliseconds
    #[arg(long, default_value_t = 500)]
    retry_delay: u64,

    /// (Optional) Custom User-Agent string
    #[arg(short = 'U', long = "UA", help = "Set custom User-Agent")]
    user_agent: Option<String>,

    /// (Optional) Custom headers, can be used multiple times (e.g. -H "Auth: XYZ")
    #[arg(short = 'H', long = "header")]
    headers: Vec<String>,

    /// (Optional) Connection timeout in seconds
    #[arg(long, default_value_t = 30)]
    connect_timeout: u64,

    /// (Optional) Accept invalid TLS/SSL certificates
    #[arg(long, short = 'k')]
    danger_accept_invalid_certs: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct ProbeResult {
    url: String,
    status: u16,
    #[serde(rename = "status-text")]
    status_text: String,
    length: usize,
    #[serde(rename = "content-type")]
    content_type: String,
    success: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    println!("🚀 URLProber - Fast JSON-only URL prober");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Pre-calculate line count for progress bar (if it's a file)
    let mut total_urls: usize = 0;
    let mut is_file = false;
    if let Some(path) = &cli.urls_file {
        if path.to_str() != Some("-") {
            is_file = true;
            if let Ok(file) = File::open(path) {
                let reader = io::BufReader::new(file);
                // Count lines quickly
                for line in reader.lines().map_while(Result::ok) {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with('#') {
                        total_urls += 1;
                    }
                }
            }
        }
    }

    if is_file {
        println!("📋 Found {} URLs to probe", total_urls);
    } else {
        println!("📋 Reading URLs from stdin... (Unknown total size)");
    }

    // Read external custom headers
    let mut default_headers = reqwest::header::HeaderMap::new();
    for header in &cli.headers {
        if let Some((k, v)) = header.split_once(':') {
            if let (Ok(key), Ok(val)) = (
                reqwest::header::HeaderName::from_bytes(k.trim().as_bytes()),
                reqwest::header::HeaderValue::from_str(v.trim()),
            ) {
                default_headers.insert(key, val);
            }
        }
    }

    // Build HTTP client with optimized settings
    let client = Arc::new(
        reqwest::Client::builder()
            .user_agent(
                cli.user_agent
                    .unwrap_or_else(|| DEFAULT_USER_AGENT.to_string()),
            )
            .default_headers(default_headers)
            .redirect(reqwest::redirect::Policy::limited(10))
            .connect_timeout(Duration::from_secs(cli.connect_timeout))
            .timeout(Duration::from_secs(cli.connect_timeout * 4))
            .danger_accept_invalid_certs(cli.danger_accept_invalid_certs)
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(std::cmp::min(cli.concurrency * 2, 50))

            .tcp_keepalive(Duration::from_secs(60))
            .tcp_nodelay(true)
            .build()?,
    );

    let semaphore = Arc::new(Semaphore::new(cli.concurrency));

    let pb = if is_file {
        ProgressBar::new(total_urls as u64)
    } else {
        ProgressBar::new_spinner()
    };

    let pb_style = if is_file {
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({per_sec}) {msg}")
            .unwrap()
            .progress_chars("=>-")
    } else {
        ProgressStyle::default_spinner()
            .template("[{elapsed_precise}] {spinner:.cyan} {pos} URLs probed ({per_sec}) {msg}")
            .unwrap()
    };
    pb.set_style(pb_style);

    println!(
        "⚡ Starting probe with {} concurrent workers...\n",
        cli.concurrency
    );

    // Setup streaming channel for URL results
    let (tx, mut rx) = mpsc::channel::<ProbeResult>(std::cmp::max(cli.concurrency * 2, 100));

    // Writer Task (Background) - writes to disk directly in streaming fashion
    let output_path = cli.output.clone();
    let writer_task = tokio::task::spawn_blocking(move || -> io::Result<(usize, usize)> {
        let file = File::create(&output_path)?;
        let mut writer = BufWriter::new(file);

        // Start JSON Array structure
        writer.write_all(b"{\n  \"results\": [\n")?;

        let mut first = true;
        let mut success_count = 0;
        let mut total_count = 0;

        while let Some(result) = rx.blocking_recv() {
            total_count += 1;
            if result.success {
                success_count += 1;
            }

            let result_json = serde_json::to_string(&result).unwrap();
            
            // Add comma before the element if it's not the first one
            if !first {
                writer.write_all(b",\n")?;
            } else {
                first = false;
            }
            writer.write_all(b"    ")?;
            writer.write_all(result_json.as_bytes())?;
        }

        // Close JSON Array
        if !first {
            writer.write_all(b"\n")?;
        }
        writer.write_all(b"  ]\n}\n")?;
        writer.flush()?;

        Ok((success_count, total_count))
    });

    let (url_tx, url_rx) = mpsc::channel::<String>(1000);
    let urls_file_clone = cli.urls_file.clone();

    // Stream URLs into worker channel
    tokio::task::spawn_blocking(move || {
        let is_stdin = urls_file_clone.as_ref().map_or(true, |p| p.to_str() == Some("-"));
        
        let reader: Box<dyn io::BufRead> = if is_stdin {
            Box::new(io::BufReader::new(io::stdin()))
        } else {
            let path = urls_file_clone.as_ref().unwrap();
            match File::open(path) {
                Ok(file) => Box::new(io::BufReader::new(file)),
                Err(_) => return,
            }
        };

        for line_result in reader.lines() {
            if let Ok(line) = line_result {
                let trimmed = line.trim().to_string();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    if url_tx.blocking_send(trimmed).is_err() {
                        break;
                    }
                }
            }
        }
    });

    let retries = cli.retries;
    let retry_delay = Duration::from_millis(cli.retry_delay);

    let url_stream = stream::unfold(url_rx, |mut rx| async move {
        let url = rx.recv().await;
        url.map(|u| (u, rx))
    });

    // Process URLs concurrently
    url_stream
        .for_each_concurrent(cli.concurrency, |url| {
            let client = Arc::clone(&client);
            let semaphore = Arc::clone(&semaphore);
            let pb = pb.clone();
            let tx = tx.clone();
            
            async move {
                let _permit = semaphore.acquire().await.unwrap();
                let result = probe_url_with_retry(&client, &url, retries, retry_delay).await;
                pb.inc(1);
                
                // Send result to writer
                let _ = tx.send(result).await;
            }
        })
        .await;

    // Drop our tx so the writer task correctly terminates the background loop
    drop(tx);
    pb.finish_with_message("Complete!");

    // Wait for file writing to securely flush and finish
    let (success_count, final_total) = writer_task.await.unwrap()?;

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Probe complete!");
    println!(
        "   Total: {} | Success: {} | Failed: {}",
        final_total, success_count, final_total - success_count
    );
    println!("   Output: {}", cli.output.display());

    Ok(())
}

/// Probe a URL with automatic retry logic for failed requests
async fn probe_url_with_retry(
    client: &reqwest::Client,
    url_str: &str,
    max_retries: usize,
    retry_delay: Duration,
) -> ProbeResult {
    let mut last_result = probe_url(client, url_str).await;

    // Only retry true network failures (success = false), avoiding retries on actual HTTP responses like 403/404
    if !last_result.success {
        for _attempt in 1..=max_retries {
            tokio::time::sleep(retry_delay).await;
            let retry_result = probe_url(client, url_str).await;

            if retry_result.success {
                return retry_result;
            }

            last_result = retry_result;
        }
    }

    last_result
}

/// Perform a single HTTP GET request and return structured result
async fn probe_url(client: &reqwest::Client, url_str: &str) -> ProbeResult {
    match client.get(url_str).send().await {
        Ok(response) => {
            let status = response.status();
            let status_code = status.as_u16();
            let status_text = status.canonical_reason().unwrap_or("Unknown").to_string();

            let content_type = response
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("N/A")
                .to_string();

            // Read body to get actual length
            let body = response.bytes().await.unwrap_or_default();
            let actual_length = body.len();

            ProbeResult {
                url: url_str.to_string(),
                status: status_code,
                status_text: format!("{} {}", status_code, status_text),
                length: actual_length,
                content_type,
                success: true, // Mark any valid HTTP server response as a successful probe
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            let (status_code, status_text) = if e.is_timeout() {
                (408, format!("Timeout: {}", error_msg))
            } else if e.is_connect() {
                if error_msg.contains("ssl")
                    || error_msg.contains("certificate")
                    || error_msg.contains("tls")
                {
                    (495, format!("SSL Error: {}", error_msg))
                } else {
                    (503, format!("Connection Error: {}", error_msg))
                }
            } else if e.is_redirect() {
                (300, format!("Redirect Loop: {}", error_msg))
            } else {
                (500, format!("Error: {}", error_msg))
            };

            ProbeResult {
                url: url_str.to_string(),
                status: status_code,
                status_text,
                length: 0,
                content_type: "N/A".to_string(),
                success: false,
            }
        }
    }
}
