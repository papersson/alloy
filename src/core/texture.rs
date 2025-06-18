use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTLDevice, MTLPixelFormat, MTLTexture, MTLTextureDescriptor, MTLTextureType, MTLTextureUsage,
};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    Rgba8,
    Bgra8,
}

impl TextureFormat {
    pub fn metal_format(&self) -> MTLPixelFormat {
        match self {
            Self::Rgba8 => MTLPixelFormat::RGBA8Unorm,
            Self::Bgra8 => MTLPixelFormat::BGRA8Unorm,
        }
    }

    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            Self::Rgba8 | Self::Bgra8 => 4,
        }
    }
}

pub struct Texture {
    pub texture: Retained<ProtocolObject<dyn MTLTexture>>,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
}

impl Texture {
    pub fn load(
        device: &ProtocolObject<dyn MTLDevice>,
        path: impl AsRef<Path>,
    ) -> Result<Self, String> {
        let path = path.as_ref();
        let image = image::open(path)
            .map_err(|e| format!("Failed to load image {}: {e}", path.display()))?;

        let rgba_image = image.to_rgba8();
        let (width, height) = rgba_image.dimensions();
        let format = TextureFormat::Rgba8;

        let descriptor = unsafe { MTLTextureDescriptor::new() };
        unsafe {
            descriptor.setPixelFormat(format.metal_format());
            descriptor.setWidth(width as usize);
            descriptor.setHeight(height as usize);
            descriptor.setUsage(MTLTextureUsage::ShaderRead);
        }

        let texture = device
            .newTextureWithDescriptor(&descriptor)
            .ok_or_else(|| "Failed to create texture".to_string())?;

        let bytes_per_row = width as usize * format.bytes_per_pixel();
        let region = objc2_metal::MTLRegion {
            origin: objc2_metal::MTLOrigin { x: 0, y: 0, z: 0 },
            size: objc2_metal::MTLSize {
                width: width as usize,
                height: height as usize,
                depth: 1,
            },
        };

        // Safety: rgba_image.as_raw() provides a valid slice of pixel data that lives
        // as long as rgba_image. The Metal API will copy this data into the texture.
        unsafe {
            let data_ptr = std::ptr::NonNull::new(rgba_image.as_raw().as_ptr().cast_mut().cast())
                .ok_or_else(|| {
                "Failed to create NonNull pointer for texture data".to_string()
            })?;

            texture.replaceRegion_mipmapLevel_withBytes_bytesPerRow(
                region,
                0,
                data_ptr,
                bytes_per_row,
            );
        }

        Ok(Self {
            texture,
            width,
            height,
            format,
        })
    }

    pub fn create_from_data(
        device: &ProtocolObject<dyn MTLDevice>,
        data: &[u8],
        width: u32,
        height: u32,
        format: TextureFormat,
    ) -> Result<Self, String> {
        let expected_size = width as usize * height as usize * format.bytes_per_pixel();
        if data.len() != expected_size {
            return Err(format!(
                "Invalid data size: expected {expected_size}, got {}",
                data.len()
            ));
        }

        let descriptor = unsafe { MTLTextureDescriptor::new() };
        unsafe {
            descriptor.setPixelFormat(format.metal_format());
            descriptor.setWidth(width as usize);
            descriptor.setHeight(height as usize);
            descriptor.setUsage(MTLTextureUsage::ShaderRead);
        }

        let texture = device
            .newTextureWithDescriptor(&descriptor)
            .ok_or_else(|| "Failed to create texture".to_string())?;

        let bytes_per_row = width as usize * format.bytes_per_pixel();
        let region = objc2_metal::MTLRegion {
            origin: objc2_metal::MTLOrigin { x: 0, y: 0, z: 0 },
            size: objc2_metal::MTLSize {
                width: width as usize,
                height: height as usize,
                depth: 1,
            },
        };

        unsafe {
            let data_ptr = std::ptr::NonNull::new(data.as_ptr().cast_mut().cast())
                .ok_or_else(|| "Failed to create NonNull pointer for texture data".to_string())?;

            texture.replaceRegion_mipmapLevel_withBytes_bytesPerRow(
                region,
                0,
                data_ptr,
                bytes_per_row,
            );
        }

        Ok(Self {
            texture,
            width,
            height,
            format,
        })
    }
}

pub struct TextureArray {
    pub texture: Retained<ProtocolObject<dyn MTLTexture>>,
    pub width: u32,
    pub height: u32,
    pub array_length: u32,
    pub format: TextureFormat,
}

impl TextureArray {
    /// Creates a new texture array from raw data
    ///
    /// # Arguments
    /// * `device` - Metal device
    /// * `data` - Flattened array of texture data (all slices concatenated)
    /// * `width` - Width of each texture
    /// * `height` - Height of each texture
    /// * `array_length` - Number of texture slices in the array
    /// * `format` - Pixel format
    ///
    /// # Errors
    /// Returns an error if texture creation fails or data size is invalid
    pub fn create_from_data(
        device: &ProtocolObject<dyn MTLDevice>,
        data: &[u8],
        width: u32,
        height: u32,
        array_length: u32,
        format: TextureFormat,
    ) -> Result<Self, String> {
        let bytes_per_slice = width as usize * height as usize * format.bytes_per_pixel();
        let expected_size = bytes_per_slice * array_length as usize;

        if data.len() != expected_size {
            return Err(format!(
                "Invalid data size for texture array: expected {expected_size}, got {}",
                data.len()
            ));
        }

        let descriptor = unsafe { MTLTextureDescriptor::new() };
        unsafe {
            descriptor.setTextureType(MTLTextureType::Type2DArray);
            descriptor.setPixelFormat(format.metal_format());
            descriptor.setWidth(width as usize);
            descriptor.setHeight(height as usize);
            descriptor.setArrayLength(array_length as usize);
            descriptor.setUsage(MTLTextureUsage::ShaderRead);
        }

        let texture = device
            .newTextureWithDescriptor(&descriptor)
            .ok_or_else(|| "Failed to create texture array".to_string())?;

        // Upload each slice
        let bytes_per_row = width as usize * format.bytes_per_pixel();

        for slice in 0..array_length {
            let slice_offset = slice as usize * bytes_per_slice;
            let slice_data = &data[slice_offset..slice_offset + bytes_per_slice];

            let region = objc2_metal::MTLRegion {
                origin: objc2_metal::MTLOrigin { x: 0, y: 0, z: 0 },
                size: objc2_metal::MTLSize {
                    width: width as usize,
                    height: height as usize,
                    depth: 1,
                },
            };

            unsafe {
                let data_ptr = std::ptr::NonNull::new(slice_data.as_ptr().cast_mut().cast())
                    .ok_or_else(|| {
                        "Failed to create NonNull pointer for texture data".to_string()
                    })?;

                texture.replaceRegion_mipmapLevel_slice_withBytes_bytesPerRow_bytesPerImage(
                    region,
                    0,
                    slice as usize,
                    data_ptr,
                    bytes_per_row,
                    0,
                );
            }
        }

        Ok(Self {
            texture,
            width,
            height,
            array_length,
            format,
        })
    }

    /// Creates an empty texture array that can be filled later
    pub fn create_empty(
        device: &ProtocolObject<dyn MTLDevice>,
        width: u32,
        height: u32,
        array_length: u32,
        format: TextureFormat,
    ) -> Result<Self, String> {
        let descriptor = unsafe { MTLTextureDescriptor::new() };
        unsafe {
            descriptor.setTextureType(MTLTextureType::Type2DArray);
            descriptor.setPixelFormat(format.metal_format());
            descriptor.setWidth(width as usize);
            descriptor.setHeight(height as usize);
            descriptor.setArrayLength(array_length as usize);
            descriptor.setUsage(MTLTextureUsage::ShaderRead);
        }

        let texture = device
            .newTextureWithDescriptor(&descriptor)
            .ok_or_else(|| "Failed to create texture array".to_string())?;

        Ok(Self {
            texture,
            width,
            height,
            array_length,
            format,
        })
    }

    /// Updates a single slice in the texture array
    pub fn update_slice(&self, slice_index: u32, data: &[u8]) -> Result<(), String> {
        if slice_index >= self.array_length {
            return Err(format!(
                "Slice index {} out of bounds (array length: {})",
                slice_index, self.array_length
            ));
        }

        let expected_size =
            self.width as usize * self.height as usize * self.format.bytes_per_pixel();
        if data.len() != expected_size {
            return Err(format!(
                "Invalid data size for slice: expected {expected_size}, got {}",
                data.len()
            ));
        }

        let bytes_per_row = self.width as usize * self.format.bytes_per_pixel();
        let region = objc2_metal::MTLRegion {
            origin: objc2_metal::MTLOrigin { x: 0, y: 0, z: 0 },
            size: objc2_metal::MTLSize {
                width: self.width as usize,
                height: self.height as usize,
                depth: 1,
            },
        };

        unsafe {
            let data_ptr = std::ptr::NonNull::new(data.as_ptr().cast_mut().cast())
                .ok_or_else(|| "Failed to create NonNull pointer for texture data".to_string())?;

            self.texture
                .replaceRegion_mipmapLevel_slice_withBytes_bytesPerRow_bytesPerImage(
                    region,
                    0,
                    slice_index as usize,
                    data_ptr,
                    bytes_per_row,
                    0,
                );
        }

        Ok(())
    }
}
