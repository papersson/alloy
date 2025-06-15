use crate::core::Texture;
use crate::math::Vec2;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLDevice, MTLTexture};

pub struct BitmapFont {
    texture: Retained<ProtocolObject<dyn MTLTexture>>,
    char_width: f32,
    char_height: f32,
    chars_per_row: u32,
}

impl BitmapFont {
    pub fn create_default(device: &ProtocolObject<dyn MTLDevice>) -> Self {
        // Create a simple 8x8 font texture with ASCII characters
        // For now, we'll create a basic monospace font
        const FONT_SIZE: u32 = 8;
        const CHARS_PER_ROW: u32 = 16;
        const TEXTURE_SIZE: u32 = FONT_SIZE * CHARS_PER_ROW;

        // Create font data - simple 8x8 bitmap font for digits 0-9
        let mut font_data = vec![0u8; (TEXTURE_SIZE * TEXTURE_SIZE * 4) as usize];

        // Define digit bitmaps (8x8 pixels each)
        let digits: [u64; 10] = [
            // 0
            0x3C66666E76663C00,
            // 1
            0x1818381818187E00,
            // 2
            0x3C66060C30607E00,
            // 3
            0x3C66061C06663C00,
            // 4
            0x060E1E667F060600,
            // 5
            0x7E607C0606663C00,
            // 6
            0x3C66607C66663C00,
            // 7
            0x7E660C1818181800,
            // 8
            0x3C66663C66663C00,
            // 9
            0x3C66663E06663C00,
        ];

        // Render digits into texture
        for (digit_idx, &bitmap) in digits.iter().enumerate() {
            let char_x = (digit_idx % CHARS_PER_ROW as usize) * FONT_SIZE as usize;
            let char_y = (digit_idx / CHARS_PER_ROW as usize) * FONT_SIZE as usize;

            for y in 0..8 {
                for x in 0..8 {
                    let bit = (bitmap >> (63 - (y * 8 + x))) & 1;
                    if bit == 1 {
                        let px = char_x + x;
                        let py = char_y + y;
                        let idx = ((py * TEXTURE_SIZE as usize + px) * 4) as usize;
                        font_data[idx] = 255; // R
                        font_data[idx + 1] = 255; // G
                        font_data[idx + 2] = 255; // B
                        font_data[idx + 3] = 255; // A
                    }
                }
            }
        }

        // Add 'F', 'P', 'S' characters
        let letters: [(usize, u64); 3] = [
            // F at position 10
            (10, 0x7E60607C60606000),
            // P at position 11
            (11, 0x7C66667C60606000),
            // S at position 12
            (12, 0x3C66603C06663C00),
        ];

        for (char_idx, bitmap) in letters {
            let char_x = (char_idx % CHARS_PER_ROW as usize) * FONT_SIZE as usize;
            let char_y = (char_idx / CHARS_PER_ROW as usize) * FONT_SIZE as usize;

            for y in 0..8 {
                for x in 0..8 {
                    let bit = (bitmap >> (63 - (y * 8 + x))) & 1;
                    if bit == 1 {
                        let px = char_x + x;
                        let py = char_y + y;
                        let idx = ((py * TEXTURE_SIZE as usize + px) * 4) as usize;
                        font_data[idx] = 255; // R
                        font_data[idx + 1] = 255; // G
                        font_data[idx + 2] = 255; // B
                        font_data[idx + 3] = 255; // A
                    }
                }
            }
        }

        let texture = Texture::create_from_data(
            device,
            &font_data,
            TEXTURE_SIZE,
            TEXTURE_SIZE,
            crate::core::TextureFormat::Rgba8,
        )
        .expect("Failed to create font texture");

        Self {
            texture: texture.texture,
            char_width: FONT_SIZE as f32,
            char_height: FONT_SIZE as f32,
            chars_per_row: CHARS_PER_ROW,
        }
    }

    pub fn texture(&self) -> &ProtocolObject<dyn MTLTexture> {
        &self.texture
    }

    pub fn get_char_uv(&self, ch: char) -> Option<(Vec2, Vec2)> {
        let char_index = match ch {
            '0'..='9' => (ch as u32 - '0' as u32) as usize,
            'F' => 10,
            'P' => 11,
            'S' => 12,
            _ => return None,
        };

        let x = (char_index % self.chars_per_row as usize) as f32;
        let y = (char_index / self.chars_per_row as usize) as f32;

        let u0 = x * self.char_width / (self.chars_per_row as f32 * self.char_width);
        let v0 = y * self.char_height / (self.chars_per_row as f32 * self.char_height);
        let u1 = (x + 1.0) * self.char_width / (self.chars_per_row as f32 * self.char_width);
        let v1 = (y + 1.0) * self.char_height / (self.chars_per_row as f32 * self.char_height);

        Some((Vec2::new(u0, v0), Vec2::new(u1, v1)))
    }

    pub fn char_size(&self) -> Vec2 {
        Vec2::new(self.char_width, self.char_height)
    }
}
