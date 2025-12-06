use image::ImageFormat;
use std::io::Cursor;
use crate::models::settings::VariantConfig;
use crate::error::AppError;

pub fn process_image(data: &[u8], config: &VariantConfig) -> Result<(Vec<u8>, String), AppError> {
    // 1. Load image
    let mut img = image::load_from_memory(data)
        .map_err(|e| AppError::InternalServerError(format!("Failed to load image: {}", e)))?;

    // 2. Resize if needed
    // 2. Resize if needed
    // Logic:
    // - If both width and height are provided (and fit wasn't cover/contain specific): assume exact resize or fit?
    //   For safety and simplicity given standard use cases (w1200), we probably want 'resize' (fit within) if one is missing, 
    //   or 'resize_exact' if both are present?
    //   Actually, standard behavior for 'width=1200, height=null' is "width 1200, auto height".
    //   Standard behavior for 'width=1200, height=800' could be "force 1200x800".

    // 2. Resize if needed
    let filter = image::imageops::FilterType::Lanczos3;
    let fit = config.fit.as_deref().unwrap_or("contain"); // Default to contain if not specified

    if let (Some(w), Some(h)) = (config.width, config.height) {
        match fit {
            "cover" | "center-crop" => {
                img = img.resize_to_fill(w, h, filter);
            },
            "fill" | "stretch" | "exact" => {
                img = img.resize_exact(w, h, filter);
            },
            _ => {
                // Default "contain" / "inside" behavior
                img = img.resize(w, h, filter);
            }
        }
    } else if let Some(w) = config.width {
        // Only width: maintain aspect ratio
        img = img.resize(w, u32::MAX, filter);
    } else if let Some(h) = config.height {
        // Only height: maintain aspect ratio
        img = img.resize(u32::MAX, h, filter);
    } else if let (Some(w), Some(h)) = (config.max_width, config.max_height) {
        // Max dimensions: fit within
        img = img.resize(w, h, filter);
    }

    // 3. Determine Output Format
    let format_str = config.format.as_deref().unwrap_or("original");
    let (output_format, mime_type) = match format_str {
        "avif" => (ImageFormat::Avif, "image/avif"),
        "webp" => (ImageFormat::WebP, "image/webp"),
        "png" => (ImageFormat::Png, "image/png"),
        "jpg" | "jpeg" => (ImageFormat::Jpeg, "image/jpeg"),
        "original" => {
            // Detect original format
            let fmt = image::guess_format(data)
                .map_err(|e| AppError::InternalServerError(format!("Failed to guess format: {}", e)))?;
            let mime = match fmt {
                ImageFormat::Avif => "image/avif",
                ImageFormat::WebP => "image/webp",
                ImageFormat::Png => "image/png",
                ImageFormat::Jpeg => "image/jpeg",
                _ => "application/octet-stream",
            };
            (fmt, mime)
        },
        _ => (ImageFormat::Jpeg, "image/jpeg"), // Default fallback
    };

    // 4. Encode with Quality
    let mut buffer = Cursor::new(Vec::new());
    
    // Note: The `image` crate's `write_to` doesn't always expose quality controls for all formats easily 
    // in the generic API, but for JPEG/WebP/AVIF it often uses defaults or we can use specific encoders.
    // For simplicity in this phase, we'll use the generic `write_to` which uses reasonable defaults,
    // but for JPEG/WebP/AVIF we can try to respect the quality setting if we use specific encoders.
    // However, `DynamicImage::write_to` is the most robust way to handle multiple formats.
    // To support quality specifically, we might need to match on format.

    match output_format {
        // For now, use default quality. To support custom quality, we'd need to use specific Encoders
        // e.g. JpegEncoder::new_with_quality(&mut buffer, quality)
        // But for simplicity and compilation, we stick to write_to with default settings.
        _ => {
            img.write_to(&mut buffer, output_format)
                .map_err(|e| AppError::InternalServerError(format!("Failed to encode image: {}", e)))?;
        }
    }

    Ok((buffer.into_inner(), mime_type.to_string()))
}
