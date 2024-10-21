pub mod jwk;

use anybuf::{Anybuf, Bufany, BufanyError};

pub struct QueryWebAuthNVerifyAuthenticateRequest<'a> {
    pub addr: String,         // 1
    pub challenge: String,    // 2
    pub rp: String,           // 3
    pub credential: &'a [u8], // 4
    pub data: &'a [u8],       // 5
}

impl QueryWebAuthNVerifyAuthenticateRequest<'_> {
    pub fn to_anybuf(&self) -> Anybuf {
        Anybuf::new()
            .append_string(1, &self.addr)
            .append_string(2, &self.challenge)
            .append_string(3, &self.rp)
            .append_bytes(4, self.credential)
            .append_bytes(5, self.data)
    }
}

pub struct QueryWebAuthNVerifyRegisterRequest<'a> {
    pub addr: String,      // 1
    pub challenge: String, // 2
    pub rp: String,        // 3
    pub data: &'a [u8],    // 4
}

impl QueryWebAuthNVerifyRegisterRequest<'_> {
    pub fn to_anybuf(&self) -> Anybuf {
        Anybuf::new()
            .append_string(1, &self.addr)
            .append_string(2, &self.challenge)
            .append_string(3, &self.rp)
            .append_bytes(4, self.data)
    }
}

pub struct QueryWebAuthNVerifyRegisterResponse {
    pub credential: Vec<u8>, // 1
}

impl QueryWebAuthNVerifyRegisterResponse {
    pub fn from_bufany(bufany: &Bufany) -> Self {
        let credential = bufany.bytes(1).unwrap_or_default();
        Self { credential }
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, BufanyError> {
        let bufany = Bufany::deserialize(slice)?;
        Ok(Self::from_bufany(&bufany))
    }
}
