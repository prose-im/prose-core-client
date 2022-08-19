use base64;
use sha1::{Digest, Sha1};

pub struct Image {
    pub data: Vec<u8>,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
}

impl Image {
    pub fn new(data: Vec<u8>, mime_type: impl Into<String>, width: u32, height: u32) -> Self {
        Image {
            data,
            mime_type: mime_type.into(),
            width,
            height,
        }
    }
}

impl Image {
    pub fn base64_string(&self) -> String {
        base64::encode(&self.data)
    }

    pub fn sha1_checksum(&self) -> String {
        let mut hasher = Sha1::new();
        hasher.update(&self.data);
        format!("{:x}", hasher.finalize())
    }
}
