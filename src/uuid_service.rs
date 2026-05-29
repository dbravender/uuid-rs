use uuid::Uuid as RawUuid;

use tonic::{Request, Response, Status};

use crate::proto::GenerateRequest;
use crate::proto::GenerateResponse;
use crate::proto::uuid_server::Uuid;

/// Generate one real `UUIDv4` response as raw 16-byte Protobuf bytes.
#[must_use]
pub fn generate_response() -> GenerateResponse {
    GenerateResponse {
        uuid: RawUuid::new_v4().into_bytes().to_vec(),
    }
}

/// Convert a raw-byte UUID response into a UUID value for client-side formatting.
pub fn uuid_from_response(response: &GenerateResponse) -> Result<RawUuid, InvalidUuidBytes> {
    let bytes: [u8; 16] = response
        .uuid
        .as_slice()
        .try_into()
        .map_err(|_| InvalidUuidBytes {
            actual_len: response.uuid.len(),
        })?;
    Ok(RawUuid::from_bytes(bytes))
}

/// Error returned when a response does not contain exactly 16 UUID bytes.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct InvalidUuidBytes {
    actual_len: usize,
}

impl InvalidUuidBytes {
    /// Return the byte length that was received.
    #[must_use]
    pub const fn actual_len(self) -> usize {
        self.actual_len
    }
}

impl std::fmt::Display for InvalidUuidBytes {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "expected 16 UUID bytes, received {}",
            self.actual_len
        )
    }
}

impl std::error::Error for InvalidUuidBytes {}

/// gRPC implementation for the UUID service.
#[derive(Debug, Default, Clone, Copy)]
pub struct UuidGenerator;

#[tonic::async_trait]
impl Uuid for UuidGenerator {
    async fn generate(
        &self,
        _request: Request<GenerateRequest>,
    ) -> Result<Response<GenerateResponse>, Status> {
        Ok(Response::new(generate_response()))
    }
}

#[cfg(test)]
mod tests {
    use uuid::{Variant, Version};

    use super::{GenerateResponse, generate_response, uuid_from_response};

    #[test]
    fn generate_response_returns_exactly_sixteen_uuid_bytes() {
        let response = generate_response();

        assert_eq!(response.uuid.len(), 16);
    }

    #[test]
    fn generate_response_returns_real_uuid_v4_bytes() {
        let response = generate_response();
        let uuid = uuid_from_response(&response).expect("response should be a UUID");

        assert_eq!(uuid.get_version(), Some(Version::Random));
        assert_eq!(uuid.get_variant(), Variant::RFC4122);
    }

    #[test]
    fn uuid_from_response_rejects_invalid_lengths() {
        let response = GenerateResponse {
            uuid: vec![1, 2, 3],
        };

        let error = uuid_from_response(&response).expect_err("three bytes is not a UUID");

        assert_eq!(error.actual_len(), 3);
        assert_eq!(error.to_string(), "expected 16 UUID bytes, received 3");
    }

    #[test]
    fn client_side_formatting_produces_canonical_uuid_string() {
        let response = GenerateResponse {
            uuid: uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8")
                .into_bytes()
                .to_vec(),
        };

        let uuid = uuid_from_response(&response).expect("valid UUID bytes");

        assert_eq!(
            uuid.hyphenated().to_string(),
            "67e55044-10b1-426f-9247-bb680e5fe0c8"
        );
    }
}
