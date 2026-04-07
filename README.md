# URLProber 🚀

A lightning-fast, zero-overhead, highly concurrent URL prober designed for massive-scale reconnaissance. Written purely in asynchronous Rust.

URLProber was specifically decoupled from standard memory-loading procedures to utilize a highly-optimized **Streaming Architecture** via `tokio` multi-producer, single-consumer (MPSC) channels. This guarantees memory complexity stays completely flat `O(1)`, allowing you to painlessly probe 100 or 10 Million URLs with virtually exact RAM footprint properties.

## Features
- **Streaming Architecture**: Direct `stdin` / `file` reading stream pipes seamlessly into async background JSON writer tasks to maintain baseline memory allocation.
- **Intelligent Retry Logic**: Implements strategic retry procedures strictly isolated to timeout & network level failure without stalling the engine.
- **Highly Concurrent**: Fully leverages system threading & Tokio’s runtime logic (`--concurrency` flag).
- **Custom Requests**: Easily attach any arbitrary HTTP headers (`-H`) or a custom User-Agent mapping (`--UA`).
- **TLS Bypass**: Optional flags to bypass invalid / missing network certificates smoothly.
- **CLI Native**: Supports pipeline tools like `cat`, `jq`, etc., out of the box.

## Installation

As this tool achieves extremely high operations bounds concurrently, ensure you build exactly with the `--release` runtime optimizer.

```bash
git clone https://github.com/v0id0x/urlprober.git
cd urlprober

# If you face NTFS OS permissions on WSL, use the custom target fix:
CARGO_TARGET_DIR=/tmp/target cargo build --release
```

## Usage

You must specify your input (via piping or file) and explicitly define your JSON output target file.

```bash
# Basic Probe using an input file
urlprober -f input_urls.txt -o results.json

# Defining Concurrency and explicit Timeouts
urlprober -f inputs.txt -o results.json --concurrency 200 --connect-timeout 10

# Stdin Piping Mode (Reads unknown length standard input pipes):
cat urls.txt | urlprober -o results.json

# Adding Custom Headers and Custom User-Agent
urlprober -f inputs.txt -o results.json -H "Authorization: Bearer XYZ" -H "X-Custom: 123" --UA "Mozilla/5.0 (Windows NT 10.0)"
```

### Options Overview
```text
  -f, --urls-file <URLS_FILE>         Input file containing URLs (one per line). Use '-' for stdin
  -o, --output <OUTPUT>               Output JSON file path
      --concurrency <CONCURRENCY>     Maximum concurrent requests [default: 20]
      --retries <RETRIES>             Number of retry attempts for failed requests [default: 2]
      --retry-delay <RETRY_DELAY>     Delay between retries in milliseconds [default: 500]
  -U, --UA <USER_AGENT>               Custom User-Agent string
  -H, --header <HEADERS>              Custom headers, can be used multiple times (e.g. -H "Auth: XYZ")
      --connect-timeout <TIMEOUT>     Connection timeout in seconds [default: 30]
  -k, --danger-accept-invalid-certs   Accept invalid TLS/SSL certificates
```

## JSON Format

Outputs continuously generated in a wrapped memory-safe JSON array format standard:
```json
{
  "results": [
    {
      "url": "https://example.com/",
      "status": 200,
      "status-text": "200 OK",
      "length": 1256,
      "content-type": "text/html; charset=UTF-8",
      "success": true
    }
  ]
}
```

## License
MIT License
