use beserial::{uvar, Serialize, Deserialize};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Header {
    pub magic: u32,
    pub ty: u16,
    pub size: u32,
    pub checksum: u32,
}

impl Header {
    pub const MAGIC: u32 = 0x4204_2042;

    pub const SIZE: usize = 14;

    pub fn new(ty: u16, size: u32) -> Self {
        Self {
            magic: Self::MAGIC,
            ty,
            size,
            checksum: 0,
        }
    }
}

impl Default for Header {
    fn default() -> Self {
        Header::new(0, 0)
    }
}
