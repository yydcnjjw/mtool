use base64::prelude::*;
use image::DynamicImage;
use mapp::anyhow;
use std::io::Cursor;

pub fn base64_image(image: DynamicImage) -> Result<String, anyhow::Error> {
    let mut buffer = Vec::new();
    image.write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::Png)?;
    Ok(format!(
        "url(\"data:image/png;base64,{}\")",
        BASE64_STANDARD.encode(buffer),
    ))
}
