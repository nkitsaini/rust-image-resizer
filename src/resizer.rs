use anyhow::Context;
use std::io::{BufWriter, Cursor};
use std::num::{NonZeroU32, ParseIntError};
use std::str::FromStr;

use axum::body::Bytes;
use image::codecs::jpeg::JpegEncoder;
use image::io::Reader as ImageReader;
use image::{ColorType, ImageEncoder};
use crate::overflow_ops::mul_div3;

use fast_image_resize as fr;
use serde::{Deserialize, Deserializer};

use tracing::{debug};

fn default_jpeg_quality() -> std::num::NonZeroU8 {
    std::num::NonZeroU8::new(100).unwrap()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageUnit(std::num::NonZeroU32);
impl ImageUnit {
    fn mul_div_l(a: NonZeroU32, b: NonZeroU32, c: NonZeroU32) -> Option<NonZeroU32> {
        let rv = mul_div3(a.get(), b.get(), c.get());
        rv.map(|x| NonZeroU32::new(x)).flatten()
    }
    fn mul_with_ratio(self, num: Self, denom: Self) -> Option<Self> {
        Some(Self(Self::mul_div_l(self.0, num.0, denom.0)?))
    }
}

impl From<NonZeroU32> for ImageUnit {
    fn from(value: NonZeroU32) -> Self{
        Self(value)
    }
}
impl FromStr for ImageUnit {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Ok(Self(u64::from_str(s)?))
        Ok(Self(std::num::NonZeroU32::new(u32::from_str(s)?).context("Value can't be zero")?))
    }
}
impl<'de> Deserialize<'de> for ImageUnit {


    fn deserialize<D>(deserializer: D) -> Result<ImageUnit, D::Error>
    where
        D: Deserializer<'de>,
    {
        use std::fmt;

        use serde::de::{self, Visitor};

        struct ImageUnitVisitor;

        impl<'de> Visitor<'de> for ImageUnitVisitor {
            type Value = ImageUnit;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an integer between 1 and 2^31")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: de::Error, {
                let value = match u32::from_str(v).context("number out of range"){
                    Ok(x) => x,
                    Err(_) => {
                        return Err(E::custom(format!("Value is out of range")))
                    }
                };
                let non_zero_value = match NonZeroU32::new(value) {
                    Some(x) => x,
                    None => {
                        return Err(E::custom(format!("Value is zero")))
                    }
                };
                Ok(ImageUnit(non_zero_value))
            }
        }
        deserializer.deserialize_str(ImageUnitVisitor)

    }
}

#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(untagged)]
pub enum ImageOutDim {
    Both{width: ImageUnit, height: ImageUnit},
    Height{height: ImageUnit},
    Width{width: ImageUnit}
}


impl ImageOutDim {
    /// Returns (width, height) if the width and height will be under the U32 range
    fn resolve(self, input_width: ImageUnit, input_height: ImageUnit) -> Option<(ImageUnit, ImageUnit)> {
        match self {
            Self::Both { width, height } => Some((width, height)),
            Self::Height { height } => Some((height.mul_with_ratio(input_width, input_height)?, height)),
            Self::Width { width } => Some((width, width.mul_with_ratio(input_height, input_width)?)),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct ImageResizeParams {

    #[serde(default = "default_jpeg_quality")]
    jpeg_quality: std::num::NonZeroU8,

    #[serde(flatten)]
    out_dim: ImageOutDim,
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

    let (dst_width, dst_height) = params.out_dim.resolve(width.into(), height.into()).context("Output dims too large")?;
    let mut dst_image = fr::Image::new(dst_width.0, dst_height.0, src_image.pixel_type());

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
            dst_width.0.get(),
            dst_height.0.get(),
            ColorType::Rgba8,
        )?;
    }
    debug!("Image converted");
    Ok(rv)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore] //only works for string values right now
    #[test]
    fn deserialization_json() {
        let value: ImageResizeParams =  serde_json::from_str("{\"width\": 10}").unwrap();
        dbg!(value);
        let value: ImageResizeParams =  serde_json::from_str("{\"height\": 10}").unwrap();
        dbg!(value);
        let value: ImageResizeParams =  serde_json::from_str("{\"height\": 10, \"width\":27}").unwrap();
        dbg!(value);
        let value: ImageResizeParams =  serde_json::from_str("{\"height\": 10, \"width\":27, \"jpeg_quality\": 27}").unwrap();
        dbg!(value);
    }


    #[ignore]
    #[test]
    fn deserialization_url() {
        let value: ImageOutDim =  dbg!(serde_urlencoded::from_str("width=10")).unwrap();
        let value: ImageResizeParams =  serde_json::from_str("width=10").unwrap();
        dbg!(value);
        let value: ImageResizeParams =  serde_json::from_str("height=10").unwrap();
        dbg!(value);
        let value: ImageResizeParams =  serde_json::from_str("height=10&width=30").unwrap();
        dbg!(value);
        let value: ImageResizeParams =  serde_json::from_str("height=10&width=30&jpeg_quality=77").unwrap();
        dbg!(value);
    }
}