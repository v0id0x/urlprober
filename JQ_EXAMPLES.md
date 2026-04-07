# URLProber - jq Command Examples

## 🎯 Quick View (Most Useful!)

```bash
# Show: status - length - url
cat results.json | jq -c '.results[] | [.status, .length, .url] | join(" - ")'

# Output example:
# "200 - 173067 - https://google.com"
# "404 - 0 - https://api.example.com/notfound"
# "200 - 5234 - http://13.13.1.110/"
```

## 📊 Filtering & Counting

```bash
# Show only successful URLs (2xx status)
jq '.results[] | select(.success == true) | .url' results.json

# Show only failed URLs
jq '.results[] | select(.success == false) | .url' results.json

# Show URLs with specific status code
jq '.results[] | select(.status == 200) | .url' results.json
jq '.results[] | select(.status == 404) | .url' results.json

# Count total URLs
jq '.results | length' results.json

# Count successful vs failed
jq '[.results[] | select(.success)] | length' results.json
jq '[.results[] | select(.success == false)] | length' results.json

# Count by status code
jq '[.results[] | .status] | group_by(.) | map({status: .[0], count: length})' results.json
```

## 📏 Size Analysis

```bash
# Show 10 largest responses
jq '.results | sort_by(.length) | reverse | .[0:10] | .[] | "\(.length) - \(.url)"' results.json

# Show 10 smallest responses
jq '.results | sort_by(.length) | .[0:10] | .[] | "\(.length) - \(.url)"' results.json

# Calculate total size of all responses
jq '[.results[].length] | add' results.json

# Average response size
jq '[.results[].length] | add / length' results.json

# Show URLs larger than 100KB
jq '.results[] | select(.length > 100000) | "\(.length) - \(.url)"' results.json
```

## 🔍 Content Type Analysis

```bash
# Group by content type
jq '[.results[] | .["content-type"]] | group_by(.) | map({type: .[0], count: length})' results.json

# Show only HTML pages
jq '.results[] | select(.["content-type"] | contains("html")) | .url' results.json

# Show only JSON APIs
jq '.results[] | select(.["content-type"] | contains("json")) | .url' results.json

# Show only images
jq '.results[] | select(.["content-type"] | contains("image")) | .url' results.json
```

## 🎨 Custom Formatting

```bash
# Custom format: URL | Status | Size
jq -r '.results[] | "\(.url) | \(.status) | \(.length) bytes"' results.json

# Only URL and status (tab-separated)
jq -r '.results[] | "\(.url)\t\(.status)"' results.json

# CSV format
jq -r '.results[] | [.url, .status, .length, .["content-type"]] | @csv' results.json

# Markdown table format
jq -r '.results[] | "| \(.url) | \(.status) | \(.length) | \(.["content-type"]) |"' results.json
```

## 🚫 Error Analysis

```bash
# Show all errors with details
jq '.results[] | select(.success == false) | {url, status, error: .["status-text"]}' results.json

# Show only SSL errors (495 status)
jq '.results[] | select(.status == 495) | .url' results.json

# Show only timeout errors (408 status)
jq '.results[] | select(.status == 408) | .url' results.json

# Show only connection errors (503 status)
jq '.results[] | select(.status == 503) | .url' results.json

# Count errors by type
jq '[.results[] | select(.success == false) | .status] | group_by(.) | map({status: .[0], count: length})' results.json
```

## 📋 Export Formats

```bash
# Export only successful URLs to file
jq -r '.results[] | select(.success == true) | .url' results.json > successful_urls.txt

# Export failed URLs to file
jq -r '.results[] | select(.success == false) | .url' results.json > failed_urls.txt

# Export as CSV
jq -r '["URL", "Status", "Length", "Content-Type"], (.results[] | [.url, .status, .length, .["content-type"]]) | @csv' results.json > results.csv

# Export summary
jq '{
  total: (.results | length),
  successful: ([.results[] | select(.success)] | length),
  failed: ([.results[] | select(.success == false)] | length),
  total_size: ([.results[].length] | add)
}' results.json
```

## 🔧 Advanced Queries

```bash
# Show URLs with status != 200
jq '.results[] | select(.status != 200) | "\(.status) - \(.url)"' results.json

# Show URLs between 500-600 status codes (server errors)
jq '.results[] | select(.status >= 500 and .status < 600) | "\(.status) - \(.url)"' results.json

# Show redirects (3xx status)
jq '.results[] | select(.status >= 300 and .status < 400) | "\(.status) - \(.url)"' results.json

# Show client errors (4xx status)
jq '.results[] | select(.status >= 400 and .status < 500) | "\(.status) - \(.url)"' results.json

# Find empty responses
jq '.results[] | select(.length == 0) | .url' results.json

# Find specific domain
jq '.results[] | select(.url | contains("example.com")) | "\(.status) - \(.url)"' results.json
```

## 💡 Pro Tips

1. **Use `-c` for compact output** (one line per result)
2. **Use `-r` for raw output** (removes quotes)
3. **Pipe to `less` for large outputs**: `jq '...' results.json | less`
4. **Save to file**: `jq '...' results.json > output.txt`
5. **Combine with grep**: `jq -r '.results[].url' results.json | grep "admin"`

## 📚 Common Workflows

### Find all admin panels
```bash
jq -r '.results[] | select(.url | contains("admin")) | "\(.status) - \(.url)"' results.json
```

### Find all 200 OK pages
```bash
jq -r '.results[] | select(.status == 200) | .url' results.json > working_urls.txt
```

### Generate statistics
```bash
echo "Total URLs: $(jq '.results | length' results.json)"
echo "Successful: $(jq '[.results[] | select(.success)] | length' results.json)"
echo "Failed: $(jq '[.results[] | select(.success == false)] | length' results.json)"
echo "Total Size: $(jq '[.results[].length] | add' results.json) bytes"
```

---

**All commands work with URLProber JSON output format!** 🎯
