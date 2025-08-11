# Gold


## Project Structure

```
src/
  main.rs           # Entry point
  storage/
    mod.rs          # Common interface
    s3.rs           # S3 implementation
    webdav.rs       # WebDAV implementation (optional)
  config.rs         # Env/config parsing
  uploader.rs       # Upload logic
.env                # Secrets/config
Cargo.toml
```

---

## Core Components

- **Config Loader:**  
  Reads secrets and settings from `.env` or CLI args.

- **Storage Interface:**  
  Trait defining upload/list operations.  
  Implementations: S3, WebDAV, etc.

- **Uploader:**  
  Handles uploading files/folders for a package version.  
  Accepts package name, version, files, and documentation.

- **CLI Entrypoint:**  
  Accepts arguments for package name, version, file paths, etc.


## S3 Directory Convention
```
/
  package-name/
    version/         # e.g., v1.2.3, v8
      file1.zip
      file2.exe
```

## Extensibility

- All storage backends implement the same trait.
- Configurable directory structure via variables.
- Easy to add new storage backends.


## CI/CD Integration

- Script is standalone, invoked via CLI.
- All config via env or args.


## Error Handling & Logging

- retry logic for transient errors.
- detailed error messages for failed uploads.
- Logging for CI/CD traceability.


## CLI API Documentation

### Usage

```
gold upload \
  <package_name> \
  <tag> \
  [<filename>... | <pattern>...] \
  [--storage <s3|webdav>] \
  [--config <KEY=VALUE>...] \
  [--help]
```

### Commands

- **upload:**  
  Uploads a new release for a package.

### Arguments

- `<package_name>`  
  Name of the software package.

- `<tag>`  
  Version string (e.g., v1.2.3, v8).

- `<filename>... | <pattern>...`  
  List of asset files ( or name patterns) to upload for the new release.  
  To define a display label for an asset, append text starting with `#` after the file name.  

- `--storage <s3|webdav>`  
  Specify the storage backend to use. Defaults to S3.
- `--config ABC=123 DEF=456...`  
  Additional configuration variables to pass to the uploader.  
  These will override any values in the `.env` file.
