use crate::wire::{ EmbeddedData, WireError };
use image::{ DynamicImage, GenericImageView, RgbImage };
use thiserror::Error;

/// Enumeration of the Steganography errors.
#[derive(Error, Debug, PartialEq)]
pub enum SteganographyError {
    #[error("The image is too small; {needed:?} pixels are needed, and the image has {has:?} pixels.")]
    ImageTooSmall { needed: usize, has: usize },
    #[error("Parsing error")]
    FormatError(#[from] WireError),
}

/// Embed data into an image.
///
/// Parameters:
/// * input_image: Input image.
/// * input: input data to embed into the image.
///
/// Retruns:
/// The updated image or a steganography error.
pub fn embed(input_image: &DynamicImage, input: EmbeddedData) -> Result<RgbImage, SteganographyError> {
    // --- Transofrm the input data into bytes ---
    let bytes: Vec<u8> = input.into();

    // --- Validate image size ---
    let (width, height) = input_image.dimensions();
    let available = (width * height) as usize;
    let needed = (bytes.len() as f64 * 8.0 / 3.0).ceil() as usize;
    if needed > available {
        return Err(SteganographyError::ImageTooSmall { needed, has: available });
    }

    // --- Iterate over the image ---
    //let mut bits = bytes.iter().chain(hash.iter()).chain(input.iter())
    let mut bits = bytes.iter()
        .flat_map(|&byte| (0..8).rev().map(move |i| (byte >> i) & 1));
    let mut rgb = input_image.to_rgb8();
    'outer: for pixel in rgb.pixels_mut() {
        for i in 0..3 {
            match bits.next() {
                Some(b) => if b == 0 { pixel[i] &= !1 } else { pixel[i] |= 1 },
                None => break 'outer,
            }
        }
    }

    // --- Result ---
    Ok(rgb)
}

/// Extract the embeded data from an image.
///
/// Parameters:
/// * input: Input image.
///
/// Returns:
///The extracted data from the image or a steganography error.
pub fn extract(input: &DynamicImage) -> Result<EmbeddedData, SteganographyError> {
    let mut buffer: Vec<u8> = Vec::new();

    // --- Get embedded data ---
    let mut byte = 0;
    let rgb = input.to_rgb8();
    for (i, pixel) in rgb.pixels().enumerate() {
        for j in 0..3 {
            let lsb = pixel[j] & 1;
            byte = byte << 1;
            byte |= lsb;
            if (i * 3 + j + 1) % 8 == 0 {
                buffer.push(byte);
                byte = 0;
            }
        }
    }

    // --- Parse embedded data ---
    let ed: EmbeddedData = buffer.try_into()?;

    // --- Result ---
    Ok(ed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{ HashAlgorithm, EncryptionAlgorithm };
    use image::ImageBuffer;

    #[test]
    fn test() {
        let img: DynamicImage = DynamicImage::ImageRgb8(ImageBuffer::new(8, 8));
        let data = b"Test".to_vec();
        let eb = EmbeddedData::new(
            HashAlgorithm::None,
            EncryptionAlgorithm::None,
            data.clone(),
        ).unwrap();
        let out = embed(&img, eb).unwrap();
        let out = DynamicImage::ImageRgb8(out);
        let result = extract(&out).unwrap();
        assert_eq!(&data[..], &result.data);
    }

    #[test]
    fn test_limit_1() {
        let img: DynamicImage = DynamicImage::ImageRgb8(ImageBuffer::new(2, 2));
        let data = b"Test".to_vec();
        let eb = EmbeddedData::new(
            HashAlgorithm::None,
            EncryptionAlgorithm::None,
            data,
        ).unwrap();
        let result = embed(&img, eb);
        assert_eq!(result.unwrap_err(), SteganographyError::ImageTooSmall { needed: 32, has: 4 });
    }

    #[test]
    fn test_limit_2() {
        let img: DynamicImage = DynamicImage::ImageRgb8(ImageBuffer::new(2, 2));
        let data = b"Test!!".to_vec();
        let eb = EmbeddedData::new(
            HashAlgorithm::None,
            EncryptionAlgorithm::None,
            data,
        ).unwrap();
        let result = embed(&img, eb);
        assert_eq!(result.unwrap_err(), SteganographyError::ImageTooSmall { needed: 38, has: 4 });
    }

    #[test]
    fn test_limit_3() {
        let img: DynamicImage = DynamicImage::ImageRgb8(ImageBuffer::new(8, 4));
        let data = b"Test".to_vec();
        let eb = EmbeddedData::new(
            HashAlgorithm::None,
            EncryptionAlgorithm::None,
            data.clone(),
        ).unwrap();
        let out = embed(&img, eb).unwrap();
        let out = DynamicImage::ImageRgb8(out);
        let result = extract(&out).unwrap();
        assert_eq!(&data[..], &result.data);
    }
}
