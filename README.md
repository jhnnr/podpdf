<div align="center">
  <h1>
    <img src="docs/logo.svg" width="38" height="38" alt="PodPdf Logo" valign="middle" style="vertical-align: middle; margin-right: 8px;">
    PodPdf
  </h1>
  <p>
    <strong>A Lightweight, Privacy-First, Desktop-Grade PDF Processing Suite in Pure Rust.</strong>
  </p>
  <p>
    <a href="#-features--tool-catalog">Features</a> &bull;
    <a href="#-quickstart">Quickstart</a> &bull;
    <a href="#-docker-deployment">Docker</a> &bull;
    <a href="#-security--api-keys">Security</a> &bull;
    <a href="docs/api.md">API Docs</a>
  </p>
</div>

---

## 📌 Overview

**PodPdf** is a lightweight, privacy-focused PDF processing web application and REST API rewritten entirely in pure Rust. It provides a complete suite of document organization, conversion, optimization, and security tools with zero external dependencies, zero metrics collection, and zero tracking.

### 🛡️ Core Guarantees

- **100% Stateless & In-Memory:** Documents are processed entirely in memory or bounded temporary buffers. Files are never retained on disk or sent to third parties.
- **Strictly Zero Telemetry:** No analytics, no pingbacks, no metrics, no crash reporters, and no external CDN dependencies. Fully offline-capable.
- **Pure Rust Performance:** Built with the [Axum](https://github.com/tokio-rs/axum) web framework, [lopdf](https://github.com/J-F-Liu/lopdf) PDF engine, and [image](https://github.com/image-rs/image) crate.
- **Self-Contained Single-Page Application (SPA):** Modern desktop-grade UI embedded directly into the standalone binary.
- **API First:** Complete programmatic HTTP REST API authenticated strictly with programmatic API keys (`X-API-Key`).

---

## 🧰 Features & Tool Catalog

PodPdf provides 22 document operations accessible via the Web UI and REST API:

### 1. Organize Pages
- **Merge PDFs:** Combine multiple PDF documents in any order into a single unified file.
- **Split PDF:** Split documents into single-page PDFs or custom ranges (e.g. `1-2, 3, 4-5`) packaged into a ZIP archive.
- **Rotate Pages:** Rotate all or specific pages by 90°, 180°, or 270°.
- **Extract Pages:** Extract selected pages into a new document.
- **Delete Pages:** Remove unwanted pages with dead object and reference pruning.
- **Reorder Pages:** Rearrange pages into any custom sequence (e.g. `3, 1, 2`).
- **Reverse Sequence:** Invert page order from last to first.
- **Crop Margins:** Trim margins (`top`, `bottom`, `left`, `right`) updating `MediaBox` and `CropBox`.
- **Page Dimensions (Scale):** Resize page dimensions to standard sizes (`A4`, `Letter`, `A3`, `Legal`).

### 2. Edit & Stamp
- **Add Watermark:** Stamp diagonal text watermarks with customizable opacity, rotation angle, and font size.
- **Page Numbering:** Insert page numbers in headers or footers with custom formatting (e.g. `Page {n} of {total}`).
- **Auto-Redact:** Redact sensitive keywords or confidential terms from content streams with `[REDACTED]`.
- **Flatten Annotations:** Flatten interactive form fields, comments, and annotations into static page content.

### 3. Convert & Export
- **Images to PDF:** Convert PNG, JPEG, WebP, and BMP images into a paginated PDF document.
- **Text to PDF:** Convert plain text or notes into a formatted, paginated PDF with soft-wrapping.
- **PDF to Text:** Extract all raw textual content from document pages.
- **Extract Images:** Extract all embedded raster images from PDF streams into a downloadable ZIP archive.

### 4. Protect & Optimize
- **Password Protect:** Encrypt documents with AES-128 security handler and granular permission flags.
- **Unlock / Decrypt:** Remove password protection from encrypted PDF files.
- **Compress & Optimize:** Deflate streams, prune dead objects, and rewrite with cross-reference object streams.

### 5. Inspect & Compare
- **Document Metadata:** Inspect, edit, or strip metadata (Title, Author, Subject, Keywords) for anonymization.
- **Compare Documents:** Compare two PDF files across metadata, file sizes, page counts, and extracted text.

---

## 🚀 Quickstart

### Option 1: Native Cargo (Rust Toolchain)

Prerequisites: [Rust 1.80+](https://www.rust-lang.org/) installed.

```bash
# Clone the repository
git clone https://github.com/jhnnr/podpdf.git
cd podpdf

# Run the application
cargo run --release
```

Open your browser at **`http://localhost:8080`**.

### Initial Login Credentials

When authentication is enabled (default), sign in using:
- **Email:** `admin@podpdf.local` *(or `admin`)*
- **Password:** `admin`

*(You can update your email address and password at any time in **Settings > Account**)*.

---

## 🐳 Docker Deployment

### Option 2: Docker Compose (Recommended)

1. Copy the example environment configuration:
   ```bash
   cp .example.env .env
   ```

2. Start the container stack:
   ```bash
   docker compose up -d
   ```

3. View live logs:
   ```bash
   docker compose logs -f
   ```

4. Stop the services:
   ```bash
   docker compose down
   ```

### Option 3: Standalone Docker Run

```bash
# Build the image locally
docker build -t podpdf .

# Run container on port 8080 with persistent data mount
docker run -d \
  --name podpdf \
  -p 8080:8080 \
  -v $(pwd)/data:/home/podpdf/data \
  podpdf
```

---

## ⚙️ Configuration

Configuration is managed solely through environment variables and the `.env` file (copy `.example.env` to `.env` to customize).

### Environment Variables (`.env`)

| Variable | Default | Description |
|---|---|---|
| `HOST_PORT` | `8080` | External host port mapped to the container (the container internally is hardened to port `8080`). |
| `MAX_PAYLOAD_SIZE` | `52428800` | Maximum upload size in bytes (50 MB default). |
| `AUTH_ENABLED` | `true` | When `true`, enforces API key authentication on `/api/v1/pdf/*`. |
| `CUSTOM_HEADERS` | `X-Content-Type-Options:nosniff;X-Frame-Options:DENY` | Semicolon-separated HTTP response headers. |
| `RUST_LOG` | `info` | Logging filter (`error`, `warn`, `info`, `debug`, `trace`). |


---

## 🔒 Security, API Keys & Settings

### Toggleable Security Mode
Under **Settings > System Info**, administrators can toggle **Security Mode**:
- **Active (Enforced):** Every programmatic request to `/api/v1/pdf/*` must supply a valid `X-API-Key`.
- **Disabled (Open API):** Endpoints are accessible without an API key.

### Programmatic API Keys
The only way to use the REST API is using a programmatic API key.
To generate an API key:
1. Open the Web UI and sign in.
2. Click **Settings** (gear icon) &rarr; **API Keys** tab.
3. Enter a label (e.g. `CI Runner`) and click **Generate**.
4. Copy the raw key (`podpdf_...`). Raw keys are never stored on disk (stored as SHA-256 hashes).

### Custom HTTP Response Headers
Inject security and compliance headers (`Content-Security-Policy`, `X-Frame-Options`, `Cache-Control`) across all responses:
- Configurable via `.env` (`CUSTOM_HEADERS`) or dynamically via **Settings > HTTP Headers**.

---

## 📡 REST API & cURL Examples

Full REST documentation is available in [**`docs/api.md`**](docs/api.md).

### Merge PDFs
```bash
curl -X POST http://localhost:8080/api/v1/pdf/merge \
  -H "X-API-Key: podpdf_YOUR_API_KEY_HERE" \
  -F "files=@document1.pdf" \
  -F "files=@document2.pdf" \
  --output merged.pdf
```

### Compress and Optimize PDF
```bash
curl -X POST http://localhost:8080/api/v1/pdf/compress \
  -H "X-API-Key: podpdf_YOUR_API_KEY_HERE" \
  -F "file=@large_document.pdf" \
  --output optimized.pdf
```

### Add Diagonal Watermark
```bash
curl -X POST http://localhost:8080/api/v1/pdf/watermark \
  -H "X-API-Key: podpdf_YOUR_API_KEY_HERE" \
  -F "file=@contract.pdf" \
  -F "text=CONFIDENTIAL" \
  -F "font_size=48" \
  -F "angle=45" \
  --output watermarked.pdf
```

### Password Protect (Encrypt)
```bash
curl -X POST http://localhost:8080/api/v1/pdf/encrypt \
  -H "X-API-Key: podpdf_YOUR_API_KEY_HERE" \
  -F "file=@financials.pdf" \
  -F "password=SecureClientPassword123" \
  --output financials_protected.pdf
```

---

## 🧪 Testing & Quality Gate

PodPdf maintains a strict quality gate with 100% passing tests and zero warnings:

```bash
# Check compilation
cargo check

# Enforce formatting
cargo fmt -- --check

# Run static analysis with warnings denied
cargo clippy --all-targets --all-features -- -D warnings

# Run entire test suite (43 unit & integration tests)
cargo test
```

---

## 📄 License

This project is open-source and distributed under the [MIT License](LICENSE).
