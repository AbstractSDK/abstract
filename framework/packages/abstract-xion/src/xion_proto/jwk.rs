use anybuf::Anybuf;

pub struct QueryValidateJwtRequest {
    pub aud: String,       // 1
    pub sub: String,       // 2
    pub sig_bytes: String, // 3
}

impl QueryValidateJwtRequest {
    pub fn to_anybuf(&self) -> Anybuf {
        Anybuf::new()
            .append_string(1, &self.aud)
            .append_string(2, &self.sub)
            .append_string(3, &self.sig_bytes)
    }
}

#[cfg(test)]
mod test {
    use cosmos_sdk_proto::traits::MessageExt;

    use super::*;

    #[test]
    fn query_validate_jwt_request() {
        let aud = "foo".to_owned();
        let sub = "bar".to_owned();
        let sig_bytes = "baz".to_owned();
        let bytes = QueryValidateJwtRequest {
            aud: aud.clone(),
            sub: sub.clone(),
            sig_bytes: sig_bytes.clone(),
        }
        .to_anybuf()
        .into_vec();
        let expected_bytes = cosmos_sdk_proto::xion::v1::jwk::QueryValidateJwtRequest {
            aud,
            sub,
            sig_bytes,
        }
        .to_bytes()
        .unwrap();

        assert_eq!(bytes, expected_bytes);
    }
}
