# Module: filemanager (src/filemanager/)

## Purpose

Provides file management: configuration, model definitions (`FileManager`,
`FileManagerCreate`, `FileManagerInfo`), repository trait, filesystem helpers for
upload path generation, unique filename creation, file type validation, and
MIME detection. Optionally includes image reading, resizing, and writing via the
`filemanager-images` feature.

Requires the `filemanager` feature.

## ADDED Requirements

### Requirement: File upload with unique filenames

`upload_file_from_path()` SHALL validate the file extension against `allowed_types`,
detect the MIME type via `tree_magic_mini`, generate a unique randomized filename
in the tenant/app/user directory structure, and return a `FileManagerCreate` record.

See `upload_file_from_path` in `src/filemanager/helpers.rs`.

#### Scenario: Allowed type passes validation
- WHEN the file extension is in `allowed_types`
- THEN `upload_file_from_path` SHALL proceed

#### Scenario: Disallowed type rejected
- WHEN the file extension is not in `allowed_types`
- THEN a 400 validation error SHALL be returned

#### Scenario: Unique filename generation
- WHEN `generate_random_filepathbuf` is called
- THEN a filename in format `{timestamp}_{random}.{ext}` SHALL be generated
- AND if the file already exists, a new random name SHALL be tried (up to 20 attempts)

### Requirement: File retrieval path

`retrieve_file_entry_path()` SHALL return the filesystem path of a stored file,
returning 404 if the file does not exist on disk.

See `retrieve_file_entry_path` in `src/filemanager/helpers.rs`.

### Requirement: Repository trait

`FileManagerRepositoryTrait` SHALL define `get`, `create`, `delete` methods for
file metadata persistence. `ImageResizeRelRepositoryTrait` (behind `filemanager-images`
feature) SHALL add `create`, `list`, `get`, `delete` for resize relationship records.

See `src/filemanager/traits.rs`.

#### Scenario: Get file entry
- WHEN `get` is called with tenant, filename, and app_source
- THEN the corresponding `FileManager` entry SHALL be returned

### Requirement: Image reading with format fallback

`open_image()` SHALL attempt to read an image in the stated format first.
If that fails, it SHALL try all other supported formats (JPEG, PNG, WebP) in sequence.
Animated WebP and animated PNG SHALL be rejected.

See `open_image` in `src/filemanager/helpers.rs` (behind `filemanager-images`).

#### Scenario: Format fallback
- WHEN an image is labeled as JPEG but is actually a PNG
- THEN `open_image` SHALL fall back to PNG decoding and return the correct format

#### Scenario: Animated WebP rejected
- WHEN a WebP image is animated
- THEN reading SHALL return an error

### Requirement: Image resizing modes

`resize_image()` SHALL support three modes: `Fit` (scale to fit within bounds),
`Fill` (scale to fill, cropping excess), `Stretch` (scale to exact dimensions).
Resizing SHALL use Lanczos3 sampling. Supported pixel formats: GRAY8, GRAYA16,
RGB24, RGBA32.

See `resize_image` in `src/filemanager/helpers.rs` (behind `filemanager-images`).

#### Scenario: Fit mode maintains aspect ratio
- WHEN a 200x100 image is resized using `Fit` to 100x100
- THEN the output SHALL be 100x50 (maintaining aspect ratio, centered in canvas)

### Requirement: Image writing

The system SHALL write JPEG (configurable quality), PNG, and WebP images via
`write_jpeg_image`, `write_png_image`, `write_webp_image`.

See `write` module in `src/filemanager/helpers.rs` (behind `filemanager-images`).
