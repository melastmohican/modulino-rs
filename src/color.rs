//! Color type for RGB LEDs.

/// Represents an RGB color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Color {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
}

impl Color {
    /// Create a new color from RGB values.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Create a color from a 24-bit RGB value.
    pub const fn from_rgb24(rgb: u32) -> Self {
        Self {
            r: ((rgb >> 16) & 0xFF) as u8,
            g: ((rgb >> 8) & 0xFF) as u8,
            b: (rgb & 0xFF) as u8,
        }
    }

    /// Convert to a 32-bit value for APA102 LEDs.
    /// Format: 0xRRGGBB00 (red in high bits, blue shifted left by 8)
    pub const fn to_apa102_data(&self) -> u32 {
        (self.b as u32) << 8 | (self.g as u32) << 16 | (self.r as u32) << 24
    }

    /// Black (off)
    pub const BLACK: Color = Color::new(0, 0, 0);

    /// Red
    pub const RED: Color = Color::new(255, 0, 0);

    /// Green
    pub const GREEN: Color = Color::new(0, 255, 0);

    /// Blue
    pub const BLUE: Color = Color::new(0, 0, 255);

    /// Yellow
    pub const YELLOW: Color = Color::new(255, 255, 0);

    /// Cyan
    pub const CYAN: Color = Color::new(0, 255, 255);

    /// Magenta
    pub const MAGENTA: Color = Color::new(255, 0, 255);

    /// White
    pub const WHITE: Color = Color::new(255, 255, 255);

    /// Orange
    pub const ORANGE: Color = Color::new(255, 165, 0);

    /// Purple
    pub const PURPLE: Color = Color::new(128, 0, 128);
}

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self::new(r, g, b)
    }
}

impl From<Color> for (u8, u8, u8) {
    fn from(color: Color) -> Self {
        (color.r, color.g, color.b)
    }
}

impl From<u32> for Color {
    fn from(rgb: u32) -> Self {
        Self::from_rgb24(rgb)
    }
}
