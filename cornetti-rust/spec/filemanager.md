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

## MODIFIED Requirements

### Requirement: In-memory image conversion

`convert_image()` SHALL decode an image from a byte slice, optionally resize it,
and encode it into a target format byte vector — all in memory, without
touching the filesystem.

When `source_format` is `ImageFormat::Unknown`, the format SHALL be auto-detected
from magic bytes (PNG: `89 50 4E 47`, JPEG: `FF D8`, WebP: `RIFF....WEBP`).
When `target_format` is `ImageFormat::Unknown`, the source format SHALL be
preserved in the output.

If `resize` is `Some` and the requested dimensions differ from the decoded
image dimensions, the image SHALL be resized (Lanczos3 sampling) before
encoding. If dimensions match, the resize step SHALL be skipped (no-op).

JPEG quality SHALL come from `resize.quality` when present, otherwise default
to 75.

Supported input formats: PNG, JPEG, WebP. Supported output formats: PNG, JPEG,
WebP. Animated PNG and animated WebP SHALL be rejected during decoding.

See `convert_image` in `src/filemanager/helpers.rs` (behind `filemanager-images`).

#### Scenario: Format auto-detection
- WHEN `source_format` is `ImageFormat::Unknown` and `input_bytes` starts with `FF D8`
- THEN the image SHALL be decoded as JPEG

#### Scenario: Same-dimension resize is a no-op
- WHEN the decoded image is 100x100 and `resize` requests 100x100
- THEN no resize operation SHALL be performed

#### Scenario: Format conversion with resize
- WHEN a 200x100 PNG image is converted to JPEG with `Fill` resize at 100x100
- THEN the output SHALL be a 100x100 JPEG with Lanczos3-scaled content

#### Scenario: Unknown output format preserves source
- WHEN `target_format` is `ImageFormat::Unknown` and the source is PNG
- THEN the output SHALL be PNG-encoded bytes

### Requirement: In-memory image decoding from bytes

`read_png_image_from_bytes()`, `read_jpeg_image_from_bytes()`, and
`read_webp_image_from_bytes()` SHALL decode the respective image formats from
in-memory byte slices, producing an `ImageReadResult`.

- PNG decoding SHALL support 8-bit and 16-bit depth (16-bit downscaled to 8-bit),
  color types Grayscale, RGB, and RGBA. Animated PNG SHALL be rejected.
- JPEG decoding SHALL support L8, L16 (downscaled to 8-bit), RGB24, and CMYK32
  (converted to RGB). All 16-bit and CMYK inputs SHALL be normalized to 8-bit
  output.
- WebP decoding SHALL determine RGBA vs RGB mode based on alpha channel presence.
  Animated WebP SHALL be rejected.

See `read` module in `src/filemanager/helpers.rs` (behind `filemanager-images`).

#### Scenario: 16-bit PNG downscaled to 8-bit
- WHEN a 16-bit grayscale PNG is decoded via `read_png_image_from_bytes`
- THEN the output pixel values SHALL be downscaled by right-shifting 8 bits

#### Scenario: CMYK JPEG converted to RGB
- WHEN a CMYK JPEG is decoded via `read_jpeg_image_from_bytes`
- THEN pixels SHALL be converted to RGB using `(255-C)*(255-K)/255` per channel

#### Scenario: Animated WebP rejected
- WHEN an animated WebP byte slice is decoded via `read_webp_image_from_bytes`
- THEN an error SHALL be returned

### Requirement: In-memory image encoding to bytes

`write_png_image_to_bytes()`, `write_jpeg_image_to_bytes()`, and
`write_webp_image_to_bytes()` SHALL encode raw pixel data into the respective
image format as an in-memory byte vector.

- PNG encoding SHALL support GRAY8, RGB24, RGBA32, and GRAYA16 modes at 8-bit depth.
- JPEG encoding SHALL support GRAY8 (Luma), RGB24 (RGB), and RGBA32 (RGBA) modes
  with configurable quality (default 75). GRAYA16 SHALL be rejected.
- WebP encoding SHALL support GRAY8 (L8), RGB24 (Rgb8), RGBA32 (Rgba8), and
  GRAYA16 (La8) modes.

See `write` module in `src/filemanager/helpers.rs` (behind `filemanager-images`).

#### Scenario: JPEG quality defaults to 75
- WHEN `write_jpeg_image_to_bytes` is called with `quality: None`
- THEN the JPEG SHALL be encoded at quality level 75

#### Scenario: GRAYA16 rejected by JPEG encoder
- WHEN `write_jpeg_image_to_bytes` is called with mode `ImageReadTypeMode::GRAYA16`
- THEN an error SHALL be returned
