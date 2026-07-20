use crate::core::errors;
use crate::core::errors::bad_request::validation_error;
use crate::core::errors::internal_server_error::generic_error;
use crate::core::helpers::common;
use crate::core::models::CornettiResult;
use crate::filemanager::confs::FileManagerConf;
use crate::filemanager::models::{FileManagerCreate, RESOURCE_TYPE_GENERIC};
use std::fs;
use std::path::{Path, PathBuf};

pub fn get_filestem_extension_str(path: &str) -> CornettiResult<(String, String)> {
    let path = Path::new(path);
    get_filestem_extension(path)
}

pub fn get_filestem_extension(path: &Path) -> CornettiResult<(String, String)> {
    let file_stem = path
        .file_stem()
        .and_then(|stem| stem.to_str().map(|s| s.to_string()))
        .ok_or_else(|| validation_error("File stem not found".into()))?;

    let extension = path
        .extension()
        .and_then(|ext| ext.to_str().map(|s| s.to_string()))
        .map_or_else(|| "".to_string(), |ext| ext.to_lowercase());

    Ok((file_stem, extension))
}

pub fn is_allowed_file_type(extension: &str, allowed_types: &[String]) -> bool {
    allowed_types
        .iter()
        .any(|t| t.eq_ignore_ascii_case(extension))
}

pub fn gen_fs_directory(
    upload_directory: &str,
    tenant_id: &str,
    app_source: &str,
    user_id: &str,
) -> String {
    format!(
        "{}/{}/{}/{}",
        upload_directory, tenant_id, app_source, user_id
    )
}

pub fn generate_random_filepathbuf(
    extension: &str,
    base_directory: &str,
    min: Option<usize>,
    max: Option<usize>,
    max_attempts: Option<usize>,
) -> CornettiResult<(PathBuf, String)> {
    let min = min.unwrap_or(8);
    let max = max.unwrap_or(15);
    let mut max_attempts = max_attempts.unwrap_or(20);

    // Ottieni timestamp corrente
    let timestamp = chrono::Utc::now().timestamp();
    let mut random_string = common::generate_random_string(min, max);
    let timestamp_str = timestamp.to_string();
    let mut filename =
        sanitize_filename::sanitize(format!("{}_{}.{}", timestamp_str, random_string, extension));

    // Costruisci il percorso completo
    let directory_path = Path::new(base_directory);
    let mut absolute_path = fs::canonicalize(directory_path)
        .unwrap_or_else(|_| directory_path.to_path_buf())
        .join(&filename);

    // Genera stringhe finché non trova una non utilizzata
    while absolute_path.exists() && max_attempts > 0 {
        random_string = common::generate_random_string(min, max);
        filename = sanitize_filename::sanitize(format!(
            "{}_{}.{}",
            timestamp_str, random_string, extension
        ));
        absolute_path = fs::canonicalize(directory_path)
            .unwrap_or_else(|_| directory_path.to_path_buf())
            .join(&filename);

        max_attempts -= 1;

        if max_attempts == 0 {
            return Err(generic_error(
                "Unable to generate a unique filename after multiple attempts!".to_string(),
            ));
        }
    }

    Ok((absolute_path, filename))
}

pub async fn retrieve_file_entry_path(
    tenant_id: &str,
    app_source: &str,
    uploader_id: &str,
    filename: &str,
    conf: &FileManagerConf,
) -> CornettiResult<std::path::PathBuf> {
    let upload_directory =
        gen_fs_directory(&conf.upload_directory, tenant_id, app_source, uploader_id);

    let file_path: std::path::PathBuf = std::path::Path::new(&upload_directory).join(filename);
    if !file_path.exists() {
        return Err(errors::not_found::resource_not_found());
    }

    Ok(file_path)
}

#[allow(clippy::too_many_arguments)]
pub fn upload_file_from_path(
    file_path: &std::path::Path,
    filename: &str,
    filesize: usize,
    allowed_types: &[String],
    upload_directory: &str,
    tenant_id: &str,
    app_source: &str,
    identity: &str,
    identity_id: &str,
    resource_type: Option<usize>,
    parent_filename: Option<String>,
) -> CornettiResult<FileManagerCreate> {
    let (filestem, extension) = get_filestem_extension_str(filename)?;

    if !is_allowed_file_type(&extension, allowed_types) {
        return Err(crate::core::errors::bad_request::validation_error(format!(
            "File name {} with type '{}' is not allowed",
            filestem, extension
        )));
    }

    let filetype = tree_magic_mini::from_filepath(file_path).unwrap_or("unknown");

    let upload_directory = gen_fs_directory(upload_directory, tenant_id, app_source, identity_id);

    let (random_filepath_buf, random_filename) = generate_random_filepathbuf(
        &extension.to_lowercase(),
        &upload_directory,
        None,
        None,
        None,
    )?;
    let random_filepath = random_filepath_buf.as_path();

    // Create directory if it does not exist
    std::fs::create_dir_all(random_filepath.parent().unwrap())?;
    if std::fs::rename(file_path, random_filepath).is_err() {
        std::fs::copy(file_path, random_filepath)?;
        std::fs::remove_file(file_path)?;
    }

    Ok(FileManagerCreate {
        app_source: app_source.to_string(),
        filename: random_filename,
        parent_filename,
        orig_filestem: filestem,
        filesize,
        filetype: filetype.to_string(),
        extension,
        uploader_id: Some(identity_id.to_string()),
        uploader_identity: Some(identity.to_string()),
        resource_type_id: resource_type.unwrap_or(RESOURCE_TYPE_GENERIC),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_filestem_extension_str_basic() {
        let (stem, ext) = get_filestem_extension_str("myfile.txt").unwrap();
        assert_eq!(stem, "myfile");
        assert_eq!(ext, "txt");
    }

    #[test]
    fn get_filestem_extension_str_jpeg() {
        let (stem, ext) = get_filestem_extension_str("photo.JPEG").unwrap();
        assert_eq!(stem, "photo");
        assert_eq!(ext, "jpeg");
    }

    #[test]
    fn get_filestem_extension_str_png() {
        let (stem, ext) = get_filestem_extension_str("image.PNG").unwrap();
        assert_eq!(stem, "image");
        assert_eq!(ext, "png");
    }

    #[test]
    fn get_filestem_extension_str_no_extension() {
        let (stem, ext) = get_filestem_extension_str("noext").unwrap();
        assert_eq!(stem, "noext");
        assert_eq!(ext, "");
    }

    #[test]
    fn get_filestem_extension_str_multiple_dots() {
        let (stem, ext) = get_filestem_extension_str("archive.tar.gz").unwrap();
        assert_eq!(stem, "archive.tar");
        assert_eq!(ext, "gz");
    }

    #[test]
    fn get_filestem_extension_str_invalid() {
        let result = get_filestem_extension_str("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, 400);
    }

    #[test]
    fn is_allowed_file_type_match() {
        let allowed = vec!["jpg".into(), "png".into(), "gif".into()];
        assert!(is_allowed_file_type("jpg", &allowed));
        assert!(is_allowed_file_type("PNG", &allowed));
        assert!(is_allowed_file_type("GIF", &allowed));
    }

    #[test]
    fn is_allowed_file_type_no_match() {
        let allowed = vec!["jpg".into(), "png".into()];
        assert!(!is_allowed_file_type("exe", &allowed));
        assert!(!is_allowed_file_type("pdf", &allowed));
    }

    #[test]
    fn is_allowed_file_type_empty_list() {
        let allowed: Vec<String> = vec![];
        assert!(!is_allowed_file_type("jpg", &allowed));
    }

    #[test]
    fn gen_fs_directory_basic() {
        let dir = gen_fs_directory("/uploads", "tenant1", "app1", "user1");
        assert_eq!(dir, "/uploads/tenant1/app1/user1");
    }

    #[test]
    fn gen_fs_directory_different_params() {
        let dir = gen_fs_directory("/tmp/files", "TENANT", "myapp", "user_42");
        assert_eq!(dir, "/tmp/files/TENANT/myapp/user_42");
    }
}

#[cfg(feature = "filemanager-images")]
pub mod images {
    use crate::filemanager::{
        helpers::images::read::read_jpeg_image,
        models::images::{
            ImageFileManagerResize, ImageFileManagerResizeMode, ImageFormat, ImageReadResult,
            ImageReadTypeMode,
        },
    };

    fn read_image(
        src: &std::path::Path,
        format: &ImageFormat,
    ) -> Result<crate::filemanager::models::images::ImageReadResult, Box<dyn std::error::Error>>
    {
        let data = match format {
            ImageFormat::Jpeg => read_jpeg_image(src),
            ImageFormat::Png => crate::filemanager::helpers::images::read::read_png_image(src),
            ImageFormat::Webp => crate::filemanager::helpers::images::read::read_webp_image(src),
            ImageFormat::Unknown => Err("Unsupported image format for reading".into()), // Handle unknown format
        }?;
        Ok(data)
    }

    pub fn open_image(
        src: &std::path::Path,
        start_image_format: &ImageFormat,
    ) -> Result<(ImageReadResult, ImageFormat), Box<dyn std::error::Error>> {
        let mut try_read_formats = vec![
            ImageFormat::Jpeg,
            ImageFormat::Png,
            ImageFormat::Webp,
            ImageFormat::Unknown,
        ];

        let mut curr_format = start_image_format.clone();
        let mut image_data = read_image(src, start_image_format);

        if image_data.is_err() {
            log::debug!(
                "Failed to read image in format {:?}, trying others...",
                start_image_format
            );

            // If the initial format fails, try other formats
            try_read_formats.remove(
                try_read_formats
                    .iter()
                    .position(|f| *f == *start_image_format)
                    .unwrap(),
            );

            let mut found = None;
            for next_format in try_read_formats {
                match read_image(src, &next_format) {
                    Ok(data) => {
                        curr_format = next_format;
                        found = Some(data);
                        break;
                    }
                    Err(_) => continue,
                }
            }

            if let Some(data) = found {
                image_data = Ok(data);
            } else {
                return Err("Unable to read image in any supported format".into());
            }
        }

        Ok((image_data.unwrap(), curr_format))
    }

    pub fn resize_image(
        image_data: &ImageReadResult,
        curr_format: &ImageFormat,
        dest: &std::path::Path,
        resize: &ImageFileManagerResize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Placeholder for image resizing logic

        let resized = match image_data.mode {
            ImageReadTypeMode::GRAY8 => match resize.mode {
                ImageFileManagerResizeMode::Fit => gray8::resize_gray8_fit(
                    &image_data.data,
                    image_data.width,
                    image_data.height,
                    resize.width,
                    resize.height,
                ),
                ImageFileManagerResizeMode::Fill => gray8::resize_gray8_fill(
                    &image_data.data,
                    image_data.width,
                    image_data.height,
                    resize.width,
                    resize.height,
                ),
                ImageFileManagerResizeMode::Stretch => gray8::resize_gray8_stretch(
                    &image_data.data,
                    image_data.width,
                    image_data.height,
                    resize.width,
                    resize.height,
                ),
            },
            ImageReadTypeMode::GRAYA16 => match resize.mode {
                ImageFileManagerResizeMode::Fit => gray16::resize_gray16_fit(
                    &image_data.data,
                    image_data.width,
                    image_data.height,
                    resize.width,
                    resize.height,
                ),
                ImageFileManagerResizeMode::Fill => gray16::resize_gray16_fill(
                    &image_data.data,
                    image_data.width,
                    image_data.height,
                    resize.width,
                    resize.height,
                ),
                ImageFileManagerResizeMode::Stretch => gray16::resize_gray16_stretch(
                    &image_data.data,
                    image_data.width,
                    image_data.height,
                    resize.width,
                    resize.height,
                ),
            },
            ImageReadTypeMode::RGB24 => match resize.mode {
                ImageFileManagerResizeMode::Fit => rgb24::resize_rgb_fit(
                    &image_data.data,
                    image_data.width,
                    image_data.height,
                    resize.width,
                    resize.height,
                ),
                ImageFileManagerResizeMode::Fill => rgb24::resize_rgb_fill(
                    &image_data.data,
                    image_data.width,
                    image_data.height,
                    resize.width,
                    resize.height,
                ),
                ImageFileManagerResizeMode::Stretch => rgb24::resize_rgb_stretch(
                    &image_data.data,
                    image_data.width,
                    image_data.height,
                    resize.width,
                    resize.height,
                ),
            },
            ImageReadTypeMode::RGBA32 => match resize.mode {
                ImageFileManagerResizeMode::Fit => rgba32::resize_rgba_fit(
                    &image_data.data,
                    image_data.width,
                    image_data.height,
                    resize.width,
                    resize.height,
                ),
                ImageFileManagerResizeMode::Fill => rgba32::resize_rgba_fill(
                    &image_data.data,
                    image_data.width,
                    image_data.height,
                    resize.width,
                    resize.height,
                ),
                ImageFileManagerResizeMode::Stretch => rgba32::resize_rgba_stretch(
                    &image_data.data,
                    image_data.width,
                    image_data.height,
                    resize.width,
                    resize.height,
                ),
            },
        }?;

        match curr_format {
            ImageFormat::Jpeg => write::write_jpeg_image(
                dest,
                resize.width,
                resize.height,
                &resized,
                image_data.mode,
                resize.quality,
            ),
            ImageFormat::Png => {
                write::write_png_image(dest, resize.width, resize.height, &resized, image_data.mode)
            }
            ImageFormat::Webp => write::write_webp_image(
                dest,
                resize.width,
                resize.height,
                &resized,
                image_data.mode,
            ),
            _ => Err("Unsupported image format for writing".into()),
        }?;

        // TODO: Add actual resize logic here using image_data and curr_format

        Ok(())
    }

    mod read {
        use std::{fs::File, io::BufReader, io::Cursor};

        use crate::filemanager::models::images::{ImageReadResult, ImageReadTypeMode};

        pub fn read_webp_image(
            file_path: &std::path::Path,
        ) -> Result<ImageReadResult, Box<dyn std::error::Error>> {
            let raw_data = std::fs::read(file_path)?;
            let mut decoder = image_webp::WebPDecoder::new(Cursor::new(raw_data))?;

            if decoder.is_animated() {
                return Err("Animated WebP images are not supported".into());
            }

            let mode = if decoder.has_alpha() {
                ImageReadTypeMode::RGBA32
            } else {
                ImageReadTypeMode::RGB24
            };

            let (width, height) = decoder.dimensions();
            let mut data = vec![
                0;
                decoder
                    .output_buffer_size()
                    .ok_or("Failed to get output buffer size")?
            ];
            decoder.read_image(&mut data)?;

            Ok(ImageReadResult {
                width: width as usize,
                height: height as usize,
                data,
                mode,
            })
        }

        pub fn read_png_image(
            file_path: &std::path::Path,
        ) -> Result<ImageReadResult, Box<dyn std::error::Error>> {
            let decoder = png::Decoder::new(BufReader::new(File::open(file_path)?));
            let mut reader = decoder.read_info()?;
            let mut buf = vec![
                0;
                reader
                    .output_buffer_size()
                    .ok_or("Failed to get output buffer size")?
            ];
            let next_frame_info = reader.next_frame(&mut buf)?;
            let bytes = &buf[..next_frame_info.buffer_size()];
            let info = reader.info();

            let mode: ImageReadTypeMode = match info.color_type {
                png::ColorType::Grayscale => ImageReadTypeMode::GRAY8,
                png::ColorType::Rgb => ImageReadTypeMode::RGB24,
                png::ColorType::Rgba => ImageReadTypeMode::RGBA32,
                _ => return Err("Unsupported PNG color type".into()),
            };

            if info.is_animated() {
                return Err("Animated PNGs are not supported".into());
            }

            let data = match info.bit_depth {
                png::BitDepth::Eight => bytes.to_vec(),
                png::BitDepth::Sixteen => {
                    // Converti da 16-bit a 8-bit
                    bytes
                        .chunks_exact(2)
                        .map(|chunk| (u16::from_be_bytes([chunk[0], chunk[1]]) >> 8) as u8)
                        .collect()
                }
                _ => return Err("Unsupported PNG bit depth".into()),
            };

            Ok(ImageReadResult {
                width: info.width as usize,
                height: info.height as usize,
                data,
                mode,
            })
        }

        pub fn read_jpeg_image(
            file_path: &std::path::Path,
        ) -> Result<ImageReadResult, Box<dyn std::error::Error>> {
            let mut reader = jpeg_decoder::Decoder::new(File::open(file_path)?);
            let img = reader.decode()?;
            let info = reader.info().ok_or("No image info found")?;

            let data = match info.pixel_format {
                jpeg_decoder::PixelFormat::L8 => img,
                jpeg_decoder::PixelFormat::L16 => {
                    // I dati L16 sono big-endian, 2 bytes per pixel
                    img.chunks_exact(2)
                        .map(|chunk| {
                            let value = u16::from_be_bytes([chunk[0], chunk[1]]);
                            // Converti da 16-bit a 8-bit (shift di 8 bit)
                            (value >> 8) as u8
                        })
                        .collect()
                }
                jpeg_decoder::PixelFormat::RGB24 => img,
                jpeg_decoder::PixelFormat::CMYK32 => img
                    .chunks_exact(4)
                    .flat_map(|chunk| {
                        // Converti CMYK (0-255) a RGB (0-255)
                        // Formula standard: RGB = (255 - CMYK) * (255 - K) / 255

                        let c = chunk[0];
                        let m = chunk[1];
                        let y = chunk[2];
                        let k = chunk[3];

                        let k_inv = 255 - k as u16;

                        let r = ((255 - c as u16) * k_inv / 255) as u8;
                        let g = ((255 - m as u16) * k_inv / 255) as u8;
                        let b = ((255 - y as u16) * k_inv / 255) as u8;

                        [r, g, b]
                    })
                    .collect(),
            };

            let mode = match info.pixel_format {
                jpeg_decoder::PixelFormat::L8 => ImageReadTypeMode::GRAY8,
                jpeg_decoder::PixelFormat::L16 => ImageReadTypeMode::GRAY8,
                jpeg_decoder::PixelFormat::RGB24 => ImageReadTypeMode::RGB24,
                jpeg_decoder::PixelFormat::CMYK32 => ImageReadTypeMode::RGB24,
            };

            Ok(ImageReadResult {
                width: info.width as usize,
                height: info.height as usize,
                data,
                mode,
            })
        }
    }

    mod write {
        use std::fs::File;

        use crate::filemanager::models::images::ImageReadTypeMode;

        pub fn write_webp_image(
            out_path: &std::path::Path,
            width: usize,
            height: usize,
            data: &[u8],
            mode: ImageReadTypeMode,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let file = File::create(out_path)?;
            let encoder = image_webp::WebPEncoder::new(file);

            let color_type = match mode {
                ImageReadTypeMode::GRAY8 => image_webp::ColorType::L8,
                ImageReadTypeMode::RGB24 => image_webp::ColorType::Rgb8,
                ImageReadTypeMode::RGBA32 => image_webp::ColorType::Rgba8,
                ImageReadTypeMode::GRAYA16 => image_webp::ColorType::La8,
            };

            encoder.encode(data, width as u32, height as u32, color_type)?;
            Ok(())
        }

        pub fn write_png_image(
            out_path: &std::path::Path,
            width: usize,
            height: usize,
            data: &[u8],
            mode: ImageReadTypeMode,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let file = File::create(out_path)?;
            let mut encoder = png::Encoder::new(file, width as u32, height as u32);

            match mode {
                ImageReadTypeMode::GRAY8 => encoder.set_color(png::ColorType::Grayscale),
                ImageReadTypeMode::RGB24 => encoder.set_color(png::ColorType::Rgb),
                ImageReadTypeMode::RGBA32 => encoder.set_color(png::ColorType::Rgba),
                ImageReadTypeMode::GRAYA16 => encoder.set_color(png::ColorType::GrayscaleAlpha),
            }

            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header()?;
            writer.write_image_data(data)?;
            writer.finish()?;
            Ok(())
        }

        pub fn write_jpeg_image(
            out_path: &std::path::Path,
            width: usize,
            height: usize,
            data: &[u8],
            mode: ImageReadTypeMode,
            quality: Option<u8>,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let quality = quality.unwrap_or(75); // Default quality
            let file = File::create(out_path)?;
            let encoder = jpeg_encoder::Encoder::new(file, quality);

            match mode {
                ImageReadTypeMode::GRAY8 => encoder.encode(
                    data,
                    width as u16,
                    height as u16,
                    jpeg_encoder::ColorType::Luma,
                )?,
                ImageReadTypeMode::RGB24 => encoder.encode(
                    data,
                    width as u16,
                    height as u16,
                    jpeg_encoder::ColorType::Rgb,
                )?,
                ImageReadTypeMode::RGBA32 => encoder.encode(
                    data,
                    width as u16,
                    height as u16,
                    jpeg_encoder::ColorType::Rgba,
                )?,
                _ => {
                    return Err("JPEG encoder does not support (gray) Alpha channels".into());
                }
            }

            Ok(())
        }
    }

    mod rgb24 {

        use resize::{Pixel::RGB8, Type::Lanczos3};
        use rgb::FromSlice;

        pub fn resize_rgb_fit(
            src: &[u8],
            src_width: usize,
            src_height: usize,
            target_width: usize,
            target_height: usize,
        ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            // Calcola scala mantenendo proporzioni
            let scale_w = target_width as f32 / src_width as f32;
            let scale_h = target_height as f32 / src_height as f32;
            let scale = scale_w.min(scale_h);

            let new_width = (src_width as f32 * scale) as usize;
            let new_height = (src_height as f32 * scale) as usize;

            // Ridimensiona
            let mut binding = vec![0u8; new_width * new_height * 3];
            let resized = binding.as_rgb_mut();

            let mut resizer =
                resize::new(src_width, src_height, new_width, new_height, RGB8, Lanczos3)?;

            resizer.resize(src.as_rgb(), resized)?;

            // Crea canvas con padding (nero)
            let mut canvas = vec![0u8; target_width * target_height * 3];

            // Calcola offset per centrare
            let x_offset = (target_width - new_width) / 2;
            let y_offset = (target_height - new_height) / 2;

            // Copia l'immagine nel canvas
            for y in 0..new_height {
                for x in 0..new_width {
                    let src_idx = y * new_width + x;
                    let dst_idx = ((y + y_offset) * target_width + (x + x_offset)) * 3;

                    let pixel = resized[src_idx];
                    canvas[dst_idx..dst_idx + 3].copy_from_slice(&[pixel.r, pixel.g, pixel.b]);
                }
            }

            Ok(canvas)
        }

        pub fn resize_rgb_stretch(
            src: &[u8],
            src_width: usize,
            src_height: usize,
            target_width: usize,
            target_height: usize,
        ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            let mut binding = vec![0u8; target_width * target_height * 3];
            let resized = binding.as_rgb_mut();

            let mut resizer = resize::new(
                src_width,
                src_height,
                target_width,
                target_height,
                RGB8,
                Lanczos3,
            )?;

            resizer.resize(src.as_rgb(), resized)?;

            Ok(binding)
        }

        pub fn resize_rgb_fill(
            src: &[u8],
            src_width: usize,
            src_height: usize,
            target_width: usize,
            target_height: usize,
        ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            // Calcola scala mantenendo proporzioni
            let scale_w = target_width as f32 / src_width as f32;
            let scale_h = target_height as f32 / src_height as f32;
            let scale: f32 = scale_w.max(scale_h);

            let new_width = (src_width as f32 * scale) as usize;
            let new_height = (src_height as f32 * scale) as usize;

            // Ridimensiona
            let mut binding = vec![0u8; new_width * new_height * 3];
            let resized = binding.as_rgb_mut();

            let mut resizer =
                resize::new(src_width, src_height, new_width, new_height, RGB8, Lanczos3)?;

            resizer.resize(src.as_rgb(), resized)?;

            // Crea canvas con padding (nero)
            let mut canvas = vec![0u8; target_width * target_height * 3];

            // Calcola offset e padding
            let x_diff = target_width.abs_diff(new_width) / 2;
            let y_diff = target_height.abs_diff(new_height) / 2;

            let offset_src_x = if target_width < new_width { x_diff } else { 0 };

            let offset_src_y = if target_height < new_height {
                y_diff
            } else {
                0
            };

            let start_x = if target_width > new_width { x_diff } else { 0 };

            let start_y = if target_height > new_height {
                y_diff
            } else {
                0
            };

            // Copia l'immagine nel canvas
            for y in start_y..target_height {
                for x in start_x..target_width {
                    let src_idx = (y + offset_src_y) * new_width + (x + offset_src_x);
                    let dst_idx = (y * target_width + x) * 3;

                    let pixel = resized[src_idx];
                    canvas[dst_idx..dst_idx + 3].copy_from_slice(&[pixel.r, pixel.g, pixel.b]);
                }
            }

            Ok(canvas)
        }
    }

    mod rgba32 {
        use resize::{Pixel::RGBA8, Type::Lanczos3};
        use rgb::FromSlice;

        pub fn resize_rgba_fit(
            src: &[u8],
            src_width: usize,
            src_height: usize,
            target_width: usize,
            target_height: usize,
        ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            // Calcola scala mantenendo proporzioni
            let scale_w = target_width as f32 / src_width as f32;
            let scale_h = target_height as f32 / src_height as f32;
            let scale = scale_w.min(scale_h);

            let new_width = (src_width as f32 * scale) as usize;
            let new_height = (src_height as f32 * scale) as usize;

            // Ridimensiona
            let mut binding = vec![0u8; new_width * new_height * 4];
            let resized = binding.as_rgba_mut();

            let mut resizer = resize::new(
                src_width, src_height, new_width, new_height, RGBA8, Lanczos3,
            )?;

            resizer.resize(src.as_rgba(), resized)?;

            // Crea canvas con padding (trasparente)
            let mut canvas = vec![0u8; target_width * target_height * 4];

            // Calcola offset per centrare
            let x_offset = (target_width - new_width) / 2;
            let y_offset = (target_height - new_height) / 2;

            // Copia l'immagine nel canvas
            for y in 0..new_height {
                for x in 0..new_width {
                    let src_idx = y * new_width + x;
                    let dst_idx = ((y + y_offset) * target_width + (x + x_offset)) * 4;

                    let pixel = resized[src_idx];
                    canvas[dst_idx..dst_idx + 4]
                        .copy_from_slice(&[pixel.r, pixel.g, pixel.b, pixel.a]);
                }
            }

            Ok(canvas)
        }

        pub fn resize_rgba_stretch(
            src: &[u8],
            src_width: usize,
            src_height: usize,
            target_width: usize,
            target_height: usize,
        ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            let mut binding = vec![0u8; target_width * target_height * 4];
            let resized = binding.as_rgba_mut();

            let mut resizer = resize::new(
                src_width,
                src_height,
                target_width,
                target_height,
                RGBA8,
                Lanczos3,
            )?;

            resizer.resize(src.as_rgba(), resized)?;

            Ok(binding)
        }

        pub fn resize_rgba_fill(
            src: &[u8],
            src_width: usize,
            src_height: usize,
            target_width: usize,
            target_height: usize,
        ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            // Calcola scala mantenendo proporzioni
            let scale_w = target_width as f32 / src_width as f32;
            let scale_h = target_height as f32 / src_height as f32;
            let scale: f32 = scale_w.max(scale_h);

            let new_width = (src_width as f32 * scale) as usize;
            let new_height = (src_height as f32 * scale) as usize;

            // Ridimensiona
            let mut binding: Vec<u8> = vec![0u8; new_width * new_height * 4];
            let resized = binding.as_rgba_mut();

            let mut resizer = resize::new(
                src_width, src_height, new_width, new_height, RGBA8, Lanczos3,
            )?;

            resizer.resize(src.as_rgba(), resized)?;

            // Crea canvas con padding (trasparente)
            let mut canvas = vec![0u8; target_width * target_height * 4];

            // Calcola offset e padding
            let x_diff = target_width.abs_diff(new_width) / 2;
            let y_diff = target_height.abs_diff(new_height) / 2;

            let offset_src_x = if target_width < new_width { x_diff } else { 0 };

            let offset_src_y = if target_height < new_height {
                y_diff
            } else {
                0
            };

            let start_x = if target_width > new_width { x_diff } else { 0 };

            let start_y = if target_height > new_height {
                y_diff
            } else {
                0
            };

            // Copia l'immagine nel canvas
            for y in start_y..target_height {
                for x in start_x..target_width {
                    let src_idx = (y + offset_src_y) * new_width + (x + offset_src_x);
                    let dst_idx = (y * target_width + x) * 4;

                    let pixel = resized[src_idx];
                    canvas[dst_idx..dst_idx + 4]
                        .copy_from_slice(&[pixel.r, pixel.g, pixel.b, pixel.a]);
                }
            }

            Ok(canvas)
        }
    }

    mod gray8 {
        use rgb::FromSlice;

        pub fn resize_gray8_fit(
            src: &[u8],
            src_width: usize,
            src_height: usize,
            target_width: usize,
            target_height: usize,
        ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            // Calcola scala mantenendo proporzioni
            let scale_w = target_width as f32 / src_width as f32;
            let scale_h = target_height as f32 / src_height as f32;
            let scale = scale_w.min(scale_h);

            let new_width = (src_width as f32 * scale) as usize;
            let new_height = (src_height as f32 * scale) as usize;

            // Ridimensiona
            let mut binding = vec![0u8; new_width * new_height];
            let resized = binding.as_gray_mut();

            let mut resizer = resize::new(
                src_width,
                src_height,
                new_width,
                new_height,
                resize::Pixel::Gray8,
                resize::Type::Lanczos3,
            )?;

            resizer.resize(src.as_gray(), resized)?;

            // Crea canvas con padding (nero)
            let mut canvas = vec![0u8; target_width * target_height];

            // Calcola offset per centrare
            let x_offset = (target_width - new_width) / 2;
            let y_offset = (target_height - new_height) / 2;

            // Copia l'immagine nel canvas
            for y in 0..new_height {
                for x in 0..new_width {
                    let src_idx = y * new_width + x;
                    let dst_idx = (y + y_offset) * target_width + (x + x_offset);

                    let pixel = resized[src_idx];
                    canvas[dst_idx] = pixel.value();
                }
            }

            Ok(canvas)
        }

        pub fn resize_gray8_stretch(
            src: &[u8],
            src_width: usize,
            src_height: usize,
            target_width: usize,
            target_height: usize,
        ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            let mut binding = vec![0u8; target_width * target_height];
            let resized = binding.as_gray_mut();

            let mut resizer = resize::new(
                src_width,
                src_height,
                target_width,
                target_height,
                resize::Pixel::Gray8,
                resize::Type::Lanczos3,
            )?;

            resizer.resize(src.as_gray(), resized)?;

            Ok(binding)
        }

        pub fn resize_gray8_fill(
            src: &[u8],
            src_width: usize,
            src_height: usize,
            target_width: usize,
            target_height: usize,
        ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            // Calcola scala mantenendo proporzioni
            let scale_w = target_width as f32 / src_width as f32;
            let scale_h = target_height as f32 / src_height as f32;
            let scale: f32 = scale_w.max(scale_h);

            let new_width = (src_width as f32 * scale) as usize;
            let new_height = (src_height as f32 * scale) as usize;

            // Ridimensiona
            let mut binding = vec![0u8; new_width * new_height];
            let resized = binding.as_gray_mut();

            let mut resizer = resize::new(
                src_width,
                src_height,
                new_width,
                new_height,
                resize::Pixel::Gray8,
                resize::Type::Lanczos3,
            )?;

            resizer.resize(src.as_gray(), resized)?;

            // Crea canvas con padding (nero)
            let mut canvas = vec![0u8; target_width * target_height];

            // Calcola offset e padding
            let x_diff = target_width.abs_diff(new_width) / 2;
            let y_diff = target_height.abs_diff(new_height) / 2;

            let offset_src_x = if target_width < new_width { x_diff } else { 0 };

            let offset_src_y = if target_height < new_height {
                y_diff
            } else {
                0
            };

            let start_x = if target_width > new_width { x_diff } else { 0 };

            let start_y = if target_height > new_height {
                y_diff
            } else {
                0
            };

            // Copia l'immagine nel canvas
            for y in start_y..target_height {
                for x in start_x..target_width {
                    let src_idx = (y + offset_src_y) * new_width + (x + offset_src_x);
                    let dst_idx = y * target_width + x;

                    let pixel = resized[src_idx];
                    canvas[dst_idx] = pixel.value();
                }
            }

            Ok(canvas)
        }
    }

    mod gray16 {

        pub fn resize_gray16_fit(
            src: &[u8],
            src_width: usize,
            src_height: usize,
            target_width: usize,
            target_height: usize,
        ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            // Calcola scala mantenendo proporzioni
            let scale_w = target_width as f32 / src_width as f32;
            let scale_h = target_height as f32 / src_height as f32;
            let scale = scale_w.min(scale_h);

            let new_width = (src_width as f32 * scale) as usize;
            let new_height = (src_height as f32 * scale) as usize;

            // Ridimensiona

            // Convert src &[u8] to &[rgb::Gray<u16>]
            let src_gray16: Vec<rgb::Gray<u16>> = src
                .chunks_exact(2)
                .map(|chunk| rgb::Gray::new(u16::from_be_bytes([chunk[0], chunk[1]])))
                .collect();

            // Prepare output buffer as [rgb::Gray<u16>]
            let mut resized_gray16 = vec![rgb::Gray::<u16>::new(0); new_width * new_height];

            let mut resizer = resize::new(
                src_width,
                src_height,
                new_width,
                new_height,
                resize::Pixel::Gray16,
                resize::Type::Lanczos3,
            )?;

            resizer.resize(&src_gray16, &mut resized_gray16)?;

            let binding: Vec<(u8, u8)> = resized_gray16
                .iter()
                .map(|px| (px.value().to_be_bytes()[0], px.value().to_be_bytes()[1]))
                .collect();
            // Convert resized_gray16 back to [u8] (big-endian)
            /*
            for (i, px) in resized_gray16.iter().enumerate() {
                let bytes = px.value().to_be_bytes();
                binding[i * 2] = bytes[0];
                binding[i * 2 + 1] = bytes[1];
            }*/

            // Crea canvas con padding (nero)
            let mut canvas = vec![0u8; target_width * target_height * 2];

            // Calcola offset per centrare
            let x_offset = (target_width - new_width) / 2;
            let y_offset = (target_height - new_height) / 2;

            // Copia l'immagine nel canvas
            for y in 0..new_height {
                for x in 0..new_width {
                    let src_idx = y * new_width + x;
                    let dst_idx = (y + y_offset) * target_width + (x + x_offset);

                    let pixel = binding[src_idx];
                    canvas[dst_idx * 2] = pixel.0;
                    canvas[dst_idx * 2 + 1] = pixel.1;
                }
            }

            Ok(canvas)
        }

        pub fn resize_gray16_stretch(
            src: &[u8],
            src_width: usize,
            src_height: usize,
            target_width: usize,
            target_height: usize,
        ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            // Convert src &[u8] to &[rgb::Gray<u16>]
            let src_gray16: Vec<rgb::Gray<u16>> = src
                .chunks_exact(2)
                .map(|chunk| rgb::Gray::new(u16::from_be_bytes([chunk[0], chunk[1]])))
                .collect();

            // Prepare output buffer as [rgb::Gray<u16>]
            let mut resized_gray16 = vec![rgb::Gray::<u16>::new(0); target_width * target_height];

            let mut resizer = resize::new(
                src_width,
                src_height,
                target_width,
                target_height,
                resize::Pixel::Gray16,
                resize::Type::Lanczos3,
            )?;

            resizer.resize(&src_gray16, &mut resized_gray16)?;

            let result: Vec<u8> = resized_gray16
                .iter()
                .flat_map(|px| px.value().to_be_bytes())
                .collect();

            Ok(result)
        }

        pub fn resize_gray16_fill(
            src: &[u8],
            src_width: usize,
            src_height: usize,
            target_width: usize,
            target_height: usize,
        ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            // Calcola scala mantenendo proporzioni
            let scale_w = target_width as f32 / src_width as f32;
            let scale_h = target_height as f32 / src_height as f32;
            let scale: f32 = scale_w.max(scale_h);

            let new_width = (src_width as f32 * scale) as usize;
            let new_height = (src_height as f32 * scale) as usize;

            // Ridimensiona

            // Convert src &[u8] to &[rgb::Gray<u16>]
            let src_gray16: Vec<rgb::Gray<u16>> = src
                .chunks_exact(2)
                .map(|chunk| rgb::Gray::new(u16::from_be_bytes([chunk[0], chunk[1]])))
                .collect();

            // Prepare output buffer as [rgb::Gray<u16>]
            let mut resized_gray16 = vec![rgb::Gray::<u16>::new(0); new_width * new_height];

            let mut resizer = resize::new(
                src_width,
                src_height,
                new_width,
                new_height,
                resize::Pixel::Gray16,
                resize::Type::Lanczos3,
            )?;

            resizer.resize(&src_gray16, &mut resized_gray16)?;

            let binding: Vec<(u8, u8)> = resized_gray16
                .iter()
                .map(|px| (px.value().to_be_bytes()[0], px.value().to_be_bytes()[1]))
                .collect();
            // Convert resized_gray16 back to [u8] (big-endian)
            /*
            for (i, px) in resized_gray16.iter().enumerate() {
                let bytes = px.value().to_be_bytes();
                binding[i * 2] = bytes[0];
                binding[i * 2 + 1] = bytes[1];
            }*/

            // Crea canvas con padding (nero)
            let mut canvas = vec![0u8; target_width * target_height * 2];

            // Calcola offset e padding
            let x_diff = target_width.abs_diff(new_width) / 2;
            let y_diff = target_height.abs_diff(new_height) / 2;

            let offset_src_x = if target_width < new_width { x_diff } else { 0 };

            let offset_src_y = if target_height < new_height {
                y_diff
            } else {
                0
            };

            let start_x = if target_width > new_width { x_diff } else { 0 };

            let start_y = if target_height > new_height {
                y_diff
            } else {
                0
            };

            for y in start_y..target_height {
                for x in start_x..target_width {
                    let src_idx = (y + offset_src_y) * new_width + (x + offset_src_x);
                    let dst_idx = (y * target_width + x) * 2;

                    let pixel = binding[src_idx];
                    canvas[dst_idx * 2] = pixel.0;
                    canvas[dst_idx * 2 + 1] = pixel.1;
                }
            }
            Ok(canvas)
        }
    }
}
