use anyhow::Context;
use std::io::{BufWriter, Cursor};
use std::num::NonZeroU32;

use axum::body::Bytes;
use image::codecs::jpeg::JpegEncoder;
use image::io::Reader as ImageReader;
use image::{ColorType, ImageEncoder};

use fast_image_resize as fr;
use serde::Deserialize;

use tracing::{debug};

fn default_jpeg_quality() -> std::num::NonZeroU8 {
    std::num::NonZeroU8::new(100).unwrap()
}

#[derive(Deserialize)]
pub struct ImageResizeParams {
    width: std::num::NonZeroU32,
    height: std::num::NonZeroU32,

    #[serde(default = "default_jpeg_quality")]
    jpeg_quality: std::num::NonZeroU8,
}

pub fn reszier(body: Bytes, params: ImageResizeParams) -> anyhow::Result<Vec<u8>> {
    debug!(bytes_len=body.len(), "Image recieved");
    let img = ImageReader::new(Cursor::new(body))
        .with_guessed_format()?
        .decode()?;

    let width = NonZeroU32::new(img.width()).context("Image too Wide")?;
    let height = NonZeroU32::new(img.height()).context("Image too Large")?;
    let mut src_image = fr::Image::from_vec_u8(
        width,
        height,
        img.to_rgba8().into_raw(),
        fr::PixelType::U8x4,
    )?;

    // Multiple RGB channels of source image by alpha channel
    // (not required for the Nearest algorithm)
    let alpha_mul_div = fr::MulDiv::default();
    alpha_mul_div.multiply_alpha_inplace(&mut src_image.view_mut())?;

    let dst_width = params.width;
    let dst_height = params.height;
    let mut dst_image = fr::Image::new(dst_width, dst_height, src_image.pixel_type());

    // Get mutable view of destination image data
    let mut dst_view = dst_image.view_mut();

    // Create Resizer instance and resize source image
    // into buffer of destination image
    let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3));
    resizer.resize(&src_image.view(), &mut dst_view)?;

    // Divide RGB channels of destination image by alpha
    alpha_mul_div.divide_alpha_inplace(&mut dst_view)?;

    let mut rv = Vec::new();
    {
        // Write destination image as PNG-file
        let mut result_buf = BufWriter::new(&mut rv);
        JpegEncoder::new_with_quality(&mut result_buf, params.jpeg_quality.into()).write_image(
            dst_image.buffer(),
            dst_width.get(),
            dst_height.get(),
            ColorType::Rgba8,
        )?;
    }
    debug!("Image converted");
    Ok(rv)
}
