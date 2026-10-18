Guidance and instructions for AI assistants working in the PodPdf repository.

## 📌 Project Overview

**PodPdf** is a lightweight, privacy-first PDF processing application written entirely in pure Rust.

- **Tech Stack:** Rust (Axum web framework, `lopdf` processing engine, `image` decoding/encoding), modern self-contained SPA web frontend, Docker.
- **Core Architecture:** Web-only (no desktop/Tauri builds), stateless backend, RESTful API endpoints under `/api/v1/pdf/*` and `/api/v1/auth/*`.
- **Privacy Guarantee:** Zero telemetry, no metrics or tracking, no external analytics or phoning home. Fully offline-capable.
- **Security & Multi-User Settings:** Built-in user authentication, password management with salted SHA-256 hashing, programmatic API keys, and authorization middleware.

## 🛠️ Build, Lint & Test Commands

Use standard Cargo toolchain commands from the repository root.

### Common Workflow

**Check code for compilation errors:**

```bash
cargo check
```

**Format all code:**

```bash
cargo fmt --all
```

**Run linting and static analysis:**

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

**Run all unit and integration tests:**

```bash
cargo test
```

**Run the local web server with API and Web UI:**

```bash
cargo run
```

### Release & Quality Gate

**Optimized release build:**

```bash
cargo build --release
```

**Pre-PR quality gate:**

```bash
cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test
```

**Generate API and Rust documentation:**

```bash
cargo doc --no-deps --open
```

## 🐳 Docker Deployment

PodPdf runs as a containerized web service exposing API endpoints and the web UI.

### Build and Run

**Build the Docker image:**

```bash
docker build -t podpdf .
```

**Run the container on port 8080:**

```bash
docker run --rm -p 8080:8080 podpdf
```

### Docker Compose Workflow

**Start services:**

```bash
docker compose up -d
```

**Follow service logs:**

```bash
docker compose logs -f
```

**Stop services:**

```bash
docker compose down
```

### GitHub Actions CI/CD (Docker Hub Publishing)

The repository includes `.github/workflows/docker.yml` for automated container builds and publishing to Docker Hub.

**Required GitHub Repository Secrets:**
- `DOCKERHUB_USERNAME`: Your Docker Hub account username.
- `DOCKERHUB_TOKEN`: A Docker Hub Personal Access Token (PAT) with Read/Write permissions.

**Automated Triggers:**
- Pushes to `master` / `main` build, tag (`latest`, commit SHA), and push to Docker Hub.
- Release tags (e.g. `v0.1.0`) build, semver tag (`v0.1.0`, `0.1`, `0`), and push.
- Pull requests build and verify the container with GitHub Actions caching (`type=gha`) without pushing.
- Can be manually triggered via GitHub Actions `workflow_dispatch`.

## 🧰 Complete Feature & Tool Catalog

All PDF tools are accessible through the web UI dashboard at `/` and programmatic REST API endpoints under `/api/v1/*`:

### 1. Organize & Page Operations
- **Merge PDFs (`POST /api/v1/pdf/merge`):** Combine multiple PDF documents into a unified file while renumbering object trees.
- **Split PDF (`POST /api/v1/pdf/split`):** Split documents into single-page PDFs or custom ranges (e.g. `1-2, 3, 4-5`) packaged into a ZIP archive.
- **Rotate Pages (`POST /api/v1/pdf/rotate`):** Rotate all or specific pages by 90°, 180°, or 270°.
- **Extract Pages (`POST /api/v1/pdf/extract`):** Extract selected pages into a new document.
- **Remove Pages (`POST /api/v1/pdf/remove`):** Delete unwanted pages from a document with reference pruning.
- **Reorder Pages (`POST /api/v1/pdf/reorder`):** Rearrange pages into any custom sequence (e.g. `3, 1, 2`).
- **Reverse Pages (`POST /api/v1/pdf/reverse`):** Invert page sequence from last to first.
- **Crop Margins (`POST /api/v1/pdf/crop`):** Trim page margins (`top`, `right`, `bottom`, `left`) adjusting `MediaBox` and `CropBox`.
- **Scale / Resize (`POST /api/v1/pdf/scale`):** Resize page dimensions to standard sizes (A4, Letter, A3, Legal).
- **Watermark (`POST /api/v1/pdf/watermark`):** Stamp diagonal text watermarks with custom opacity, rotation angle, and font size.
- **Page Numbers (`POST /api/v1/pdf/page-numbers`):** Insert page numbers in headers or footers with customizable format templates (e.g. `Page {n} of {total}`).

### 2. Convert & Media Operations
- **Images to PDF (`POST /api/v1/pdf/images-to-pdf`):** Convert PNG, JPEG, WebP, and BMP images into multi-page PDF documents.
- **Text to PDF (`POST /api/v1/pdf/text-to-pdf`):** Convert plain text into formatted, paginated PDF pages with automated line wrapping.
- **PDF to Text (`POST /api/v1/pdf/pdf-to-text`):** Extract raw textual content from document pages.
- **Extract Images (`POST /api/v1/pdf/extract-images`):** Extract all embedded raster images from PDF streams into a downloadable ZIP archive.

### 3. Security & Optimization
- **Password Protect (`POST /api/v1/pdf/encrypt`):** Encrypt documents with AES-128 security handler and granular permission flags.
- **Unlock / Decrypt (`POST /api/v1/pdf/decrypt`):** Remove password protection from encrypted documents.
- **Flatten PDF (`POST /api/v1/pdf/flatten`):** Flatten interactive annotations, comments, and form fields into static page content.
- **Auto-Redact (`POST /api/v1/pdf/redact`):** Redact sensitive terms or confidential phrases from page content streams.
- **Compress & Optimize (`POST /api/v1/pdf/compress`):** Deflate streams, prune dead objects, and rewrite with cross-reference object streams.

### 4. Metadata & Analysis
- **Inspect Metadata (`POST /api/v1/pdf/metadata/read`):** Retrieve document title, author, subject, keywords, creation date, and page count.
- **Update Metadata (`POST /api/v1/pdf/metadata/update`):** Modify document information dictionary fields.
- **Strip Metadata (`POST /api/v1/pdf/metadata/strip`):** Remove all identification metadata for anonymization.
- **Compare PDFs (`POST /api/v1/pdf/compare`):** Compare two PDF documents across metadata, page counts, file sizes, and extracted text content.

### 5. Authentication, Settings & API Keys
- **Login Session (`POST /api/v1/auth/login`):** Authenticate with username and password, returning a secure session token.
- **Logout Session (`POST /api/v1/auth/logout`):** Invalidate an active session token.
- **Auth Status (`GET /api/v1/auth/status`):** Retrieve current login state and user profile.
- **Change Credentials (`POST /api/v1/auth/change-credentials`):** Update username and password with secure salted SHA-256 hashing.
- **List API Keys (`GET /api/v1/auth/api-keys`):** View all active programmatic API keys.
- **Create API Key (`POST /api/v1/auth/api-keys`):** Generate a new API key (`podpdf_...`) with a custom label.
- **Revoke API Key (`DELETE /api/v1/auth/api-keys/{id}`):** Revoke an existing API key.
- **System Information (`GET /api/v1/system/info`):** View application version, offline status, and telemetry guarantees.

## 🎨 Desktop-Grade Web Frontend Architecture

The embedded Single-Page Application (served at `/`) provides a responsive, desktop-grade workspace:

- **Instant Omnisearch:** Header search input filtering tools in real time.
- **Category Pills Navigation:** Top horizontal category bar (`All Tools`, `Favorites`, `Organize & Pages`, `Convert & Media`, `Security & Permissions`, `Metadata & Compare`).
- **Card Favorites:** Star icon on cards to pin favorite tools saved in `localStorage`.
- **Multi-File Queue Workspace:** Drag-and-drop dropzone with file ordering controls (`▲` / `▼`), file size indicators, and removal.
- **Settings Modal Dialog:** Dedicated tabs for Account Management, API Key generation/copying/revocation, and System Information.
- **Login Dialog:** Sign-in and session termination modal.
- **Theme Switcher:** Dark/Light mode theme toggle.
- **Zero Telemetry Guarantee:** Strictly self-contained vanilla HTML/CSS/JS with zero external CDN dependencies.

## 📐 Architecture & Principles

### 1. Privacy & Telemetry
- **Strictly No Telemetry:** No network calls to external analytical services, crash reporters, pingbacks, or usage trackers.
- **Self-Contained Frontend:** Embedded into the binary with zero CDN dependencies.

### 2. Stateless Engine & Memory Safety
- PDF transformations are processed entirely in memory or using bounded temporary files.
- Bounded payload limits (`max_payload_size`) prevent memory exhaustion.
- Temporary buffers are dropped immediately upon request completion.

### 3. Modular Code Structure
- **Domain Layer (`src/pdf/`):** Pure domain logic partitioned into focused modules (`merge`, `split`, `rotate`, `pages`, `transform`, `watermark`, `convert`, `security`, `optimize`, `compare`, `metadata`).
- **Auth Layer (`src/auth/`):** User management, password hashing, API keys, and session management.
- **Web API Layer (`src/api/`):** Thin Axum route handlers converting HTTP requests to domain operations.
- **Middleware (`src/api/middleware.rs`):** Authentication enforcement when `auth_enabled = true`.
- **Error Handling (`src/error.rs`):** Strongly-typed `thiserror` enums converting domain errors to JSON responses with proper HTTP status codes.

## 📝 Commenting & Code Style Rules

- **Contract (`///`):** Document preconditions, return types, and invariant guarantees for public functions and types.
- **Why & Hazards:** Document non-obvious PDF specification constraints, cross-reference quirks, or critical ordering.
- **No Filler:** Delete comments that restate signatures or narrate code steps.
- **No Fixed Waits:** Never use `sleep` in tests; use deterministic assertions.
