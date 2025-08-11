# Gold - Release Management Tool

Gold is a command-line tool for uploading software release assets to cloud storage backends like S3.

## Features

- [x] Upload files to S3-compatible storage (AWS S3, MinIO, etc.)
- [x] Glob pattern support for file selection (`*.zip`, `dist/*`, `**/*.exe`)
- [x] Content type detection for various file formats
- [x] Duplicate file detection and skipping
- [x] Custom display names for files
- [x] Configuration via environment variables or command-line overrides
- [x] Comprehensive error handling and validation

## Installation

```bash
cargo install --path .
```

## Configuration

Gold requires the following environment variables:

- `ACCESS_KEY` - S3 access key ID
- `SECRET_KEY` - S3 secret access key
- `BUCKET_NAME` - S3 bucket name
- `AWS_REGION` - AWS region (default: us-east-1)
- `S3_ENDPOINT` - Custom S3 endpoint (optional, for MinIO or other S3-compatible services)

### Example .env file

```env
ACCESS_KEY=your_access_key_here
SECRET_KEY=your_secret_key_here
BUCKET_NAME=my-releases-bucket
AWS_REGION=us-west-2
S3_ENDPOINT=http://localhost:9000
```

## Usage

### Basic Usage

Upload a single file:
```bash
gold upload myapp v1.0.0 target/release/myapp.exe
```

Upload multiple files:
```bash
gold upload myapp v1.0.0 target/release/myapp.exe README.md
```

### Glob Patterns

Upload all ZIP files in a directory:
```bash
gold upload myapp v1.0.0 "dist/*.zip"
```

Upload all files recursively:
```bash
gold upload myapp v1.0.0 "dist/**/*"
```

Upload files with specific extensions:
```bash
gold upload myapp v1.0.0 "*.{exe,zip,tar.gz}"
```

### Custom Display Names

You can specify custom display names for files using the `#` separator:
```bash
gold upload myapp v1.0.0 "target/release/myapp.exe#Main Application"
```

### Configuration Overrides

Override configuration values from the command line:
```bash
gold upload myapp v1.0.0 file.zip --config BUCKET_NAME=different-bucket --config AWS_REGION=eu-west-1
```

## File Organization

Files are uploaded to S3 with the following key structure:
```
{package_name}/{tag}/{filename}
```

For example:
- Package: `myapp`
- Tag: `v1.0.0` 
- File: `myapp.exe`
- S3 Key: `myapp/v1.0.0/myapp.exe`

## Supported File Types

Gold automatically detects content types for common file formats:

- Archives: `.zip`, `.tar`, `.gz`
- Executables: `.exe`, `.msi`, `.dmg`, `.deb`, `.rpm`
- Documents: `.json`, `.txt`, `.md`
- Default: `application/octet-stream`

## Error Handling

Gold provides detailed error messages for common issues:

- Missing or invalid configuration
- Network connectivity problems
- File not found errors
- Invalid glob patterns
- S3 permission issues

## Examples

### Uploading a Release

```bash
# Set up environment
export ACCESS_KEY=your_key
export SECRET_KEY=your_secret
export BUCKET_NAME=releases
export AWS_REGION=us-east-1

# Upload release assets
gold upload myproject v2.1.0 \
  "dist/myproject-*.zip" \
  "docs/README.md#Documentation" \
  "CHANGELOG.md"
```

### Using with MinIO

```bash
# Configure for MinIO
export ACCESS_KEY=minioaccess
export SECRET_KEY=miniosecret
export BUCKET_NAME=releases
export AWS_REGION=us-east-1
export S3_ENDPOINT=http://localhost:9000

gold upload myapp v1.0.0 "*.zip"
```

### CI/CD Integration

```yaml
# GitHub Actions example
- name: Upload Release Assets
  env:
    ACCESS_KEY: ${{ secrets.S3_ACCESS_KEY }}
    SECRET_KEY: ${{ secrets.S3_SECRET_KEY }}
    BUCKET_NAME: my-releases
    AWS_REGION: us-west-2
  run: |
    gold upload ${{ github.event.repository.name }} ${{ github.ref_name }} \
      "dist/*.zip" \
      "dist/*.tar.gz" \
      "README.md#Documentation"
```

## Development

### Running Tests

```bash
cargo test
```

### Building

```bash
cargo build --release
```

## License

This project is licensed under the MIT License.
