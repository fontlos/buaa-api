//! Self-implemented CRC, use to make dependencies minimize,
//! and the performance gap is negligible for the small amount of data we pass on

static CRC32_TABLE: [u32; 256] = generate_crc32_table();

const fn generate_crc32_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let polynomial = 0xEDB88320;

    let mut i = 0;
    while i < 256 {
        let mut crc = i as u32;
        let mut j = 0;
        while j < 8 {
            if crc & 1 == 1 {
                crc = (crc >> 1) ^ polynomial;
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

/// Crc32 instance
pub struct Crc32 {
    value: u32,
}

impl Crc32 {
    /// Create a new Crc32 instance
    pub fn new() -> Self {
        Self { value: 0xFFFFFFFF }
    }

    /// Update the CRC32 value with new data.
    pub fn update(&mut self, data: &[u8]) {
        for &byte in data {
            let table_index = ((self.value ^ byte as u32) & 0xFF) as usize;
            self.value = (self.value >> 8) ^ CRC32_TABLE[table_index];
        }
    }

    /// Finalize the CRC32 computation and return the digest.
    pub fn finalize(&self) -> u32 {
        !self.value
    }

    /// Compute the CRC32 of the input data.
    pub fn digest(data: &[u8]) -> u32 {
        let mut crc = Crc32::new();
        crc.update(data);
        crc.finalize()
    }
}
