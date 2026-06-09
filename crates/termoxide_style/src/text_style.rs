#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FontStyle(pub u8);

impl FontStyle {
    pub const BLINK: Self = Self(0b0000_1000);
    pub const BOLD: Self = Self(0b0000_0001);
    pub const DIM: Self = Self(0b0010_0000);
    pub const ITALIC: Self = Self(0b0000_0010);
    pub const NORMAL: Self = Self(0b0000_0000);
    pub const STRIKETHROUGH: Self = Self(0b0001_0000);
    pub const UNDERLINE: Self = Self(0b0000_0100);

    pub const fn with(self, other: Self) -> Self { Self(self.0 | other.0) }

    pub const fn without(self, other: Self) -> Self { Self(self.0 & !other.0) }

    pub const fn has(self, other: Self) -> bool { self.0 & other.0 == other.0 }

    pub const fn has_any(self, other: Self) -> bool { self.0 & other.0 != 0 }

    pub const fn is_normal(self) -> bool { self.0 == 0 }
}

impl std::ops::BitOr for FontStyle {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self { self.with(rhs) }
}
impl std::ops::BitAnd for FontStyle {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self { Self(self.0 & rhs.0) }
}
impl std::ops::BitOrAssign for FontStyle {
    fn bitor_assign(&mut self, rhs: Self) { self.0 |= rhs.0; }
}
