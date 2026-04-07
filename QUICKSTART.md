# URLProber - Quick Start Guide

## 🚀 Run (RECOMMENDED)

For your 1647 URLs with internal IPs:

```bash
cd rsaver/urlprober

# Method 1: cargo run --release (RECOMMENDED - auto-builds)
cargo run --release -- -f ../../urls.txt -o results.json -k

# Method 2: Build once, use binary
cargo build --release
./target/release/urlprober -f ../../urls.txt -o results.json -k

# Method 3: Helper script
./urlprober.sh -f ../../urls.txt -o results.json -k
```

⚠️ **CRITICAL**: Always use `--release` flag! Debug builds are 10-50x slower!

### What this does:
- `-f ../../urls.txt` - Input file with URLs
- `-o results.json` - Output JSON file
- `-k` - Accept invalid SSL certificates (MUST for internal IPs)

### Expected:
- Time: ~35-40 seconds (with --release!)
- Output: Single `results.json` file with all data
- Console: Real-time progress
- Rate: 50 req/sec (default)

## 📊 View Results

```bash
# Quick view: status - length - url (RECOMMENDED)
cat results.json | jq -c '.results[] | [.status, .length, .url] | join(" - ")'

# Pretty print JSON
cat results.json | jq '.'

# Count successful vs failed
cat results.json | jq '[.results[] | select(.success)] | length'

# Show only URLs with 200 status
cat results.json | jq '.results[] | select(.status == 200) | .url'

# Show failed URLs with errors
cat results.json | jq '.results[] | select(.success == false) | {url, status}'

# Show largest responses
jq '.results | sort_by(.length) | reverse | .[0:10]' results.json
```

## 🔧 Options

```bash
# Even faster (100 req/sec)
cargo run --release -- -f urls.txt -o results.json -k --rate-limit 100

# Slower but more patient (for slow servers)
cargo run --release -- -f urls.txt -o results.json -k --rate-limit 10 --connect-timeout 60

# Custom User-Agent
cargo run --release -- -f urls.txt -o results.json -k -u "MyBot/1.0"
```

## ✅ Output JSON Structure

```json
{
  "results": [
    {
      "url": "https://example.com",           // The URL probed
      "status": 200,                           // HTTP status code
      "status-text": "200 OK",                 // Status code + text
      "length": 1234,                          // Response size in bytes
      "content-type": "text/html; charset=UTF-8",  // Content-Type header
      "success": true                          // true if 2xx status
    }
  ]
}
```

## 💡 Pro Tips

1. **Always use `--release`**: `cargo run --release` or use binary directly
2. **Always use `-k`** for internal IPs (13.13.x.x)
3. **Default is already fast**: 50 req/sec
4. **Even faster**: Use `--rate-limit 100` for 100 req/sec
5. **Process with jq**: Super powerful JSON filtering
6. **No files clutter**: Only one JSON file is created!

## 🐛 Common Mistakes

❌ **WRONG**: `cargo run -- -f urls.txt -o out.json -k`
   (This uses debug build - VERY SLOW!)

✅ **CORRECT**: `cargo run --release -- -f urls.txt -o out.json -k`
   (This uses release build - FAST!)

---

**Ready to scan 1647 URLs in ~35-40 seconds! 🎯**
