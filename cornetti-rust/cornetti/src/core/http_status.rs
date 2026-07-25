use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum HttpStatus {
    Continue = 100,
    SwitchingProtocols = 101,
    Processing = 102,
    EarlyHints = 103,

    Ok = 200,
    Created = 201,
    Accepted = 202,
    NonAuthoritativeInformation = 203,
    NoContent = 204,
    ResetContent = 205,
    PartialContent = 206,
    MultiStatus = 207,
    AlreadyReported = 208,
    ImUsed = 226,

    MultipleChoices = 300,
    MovedPermanently = 301,
    Found = 302,
    SeeOther = 303,
    NotModified = 304,
    UseProxy = 305,
    TemporaryRedirect = 307,
    PermanentRedirect = 308,

    BadRequest = 400,
    Unauthorized = 401,
    PaymentRequired = 402,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    ProxyAuthenticationRequired = 407,
    RequestTimeout = 408,
    Conflict = 409,
    Gone = 410,
    LengthRequired = 411,
    PreconditionFailed = 412,
    PayloadTooLarge = 413,
    UriTooLong = 414,
    UnsupportedMediaType = 415,
    RangeNotSatisfiable = 416,
    ExpectationFailed = 417,
    ImATeapot = 418,
    MisdirectedRequest = 421,
    UnprocessableEntity = 422,
    Locked = 423,
    FailedDependency = 424,
    TooEarly = 425,
    UpgradeRequired = 426,
    PreconditionRequired = 428,
    TooManyRequests = 429,
    RequestHeaderFieldsTooLarge = 431,
    UnavailableForLegalReasons = 451,

    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    HttpVersionNotSupported = 505,
    VariantAlsoNegotiates = 506,
    InsufficientStorage = 507,
    LoopDetected = 508,
    NotExtended = 510,
    NetworkAuthenticationRequired = 511,
}

impl HttpStatus {
    pub fn as_u16(self) -> u16 {
        self as u16
    }

    pub fn from_u16(code: u16) -> Option<Self> {
        Some(match code {
            100..=511 => unsafe { std::mem::transmute::<u16, Self>(code) },
            _ => return None,
        })
    }

    pub fn is_informational(self) -> bool {
        (100..200).contains(&self.as_u16())
    }

    pub fn is_success(self) -> bool {
        (200..300).contains(&self.as_u16())
    }

    pub fn is_redirection(self) -> bool {
        (300..400).contains(&self.as_u16())
    }

    pub fn is_client_error(self) -> bool {
        (400..500).contains(&self.as_u16())
    }

    pub fn is_server_error(self) -> bool {
        (500..600).contains(&self.as_u16())
    }
}

impl From<HttpStatus> for u16 {
    fn from(s: HttpStatus) -> u16 {
        s.as_u16()
    }
}

impl From<u16> for HttpStatus {
    fn from(code: u16) -> Self {
        Self::from_u16(code)
            .unwrap_or_else(|| panic!("invalid HTTP status code: {}", code))
    }
}

impl Serialize for HttpStatus {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(self.as_u16())
    }
}

impl<'de> Deserialize<'de> for HttpStatus {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let code = u16::deserialize(deserializer)?;
        HttpStatus::from_u16(code)
            .ok_or_else(|| serde::de::Error::custom(format!("invalid HTTP status code: {}", code)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_u16_valid_codes() {
        assert_eq!(HttpStatus::from_u16(200), Some(HttpStatus::Ok));
        assert_eq!(HttpStatus::from_u16(404), Some(HttpStatus::NotFound));
        assert_eq!(HttpStatus::from_u16(500), Some(HttpStatus::InternalServerError));
    }

    #[test]
    fn from_u16_invalid_code() {
        assert_eq!(HttpStatus::from_u16(999), None);
    }

    #[test]
    fn as_u16_roundtrip() {
        let status = HttpStatus::Conflict;
        assert_eq!(HttpStatus::from_u16(status.as_u16()), Some(status));
    }

    #[test]
    fn is_server_error() {
        assert!(HttpStatus::InternalServerError.is_server_error());
        assert!(!HttpStatus::NotFound.is_server_error());
    }

    #[test]
    fn from_inner_u16_infallible() {
        assert_eq!(HttpStatus::from(200u16), HttpStatus::Ok);
        assert_eq!(HttpStatus::from(404u16), HttpStatus::NotFound);
    }

    #[test]
    #[should_panic(expected = "invalid HTTP status code: 999")]
    fn from_inner_u16_invalid_panics() {
        let _ = HttpStatus::from(999u16);
    }

    #[test]
    fn all_variants_have_correct_discriminant() {
        assert_eq!(HttpStatus::Ok as u16, 200);
        assert_eq!(HttpStatus::NotFound as u16, 404);
        assert_eq!(HttpStatus::InternalServerError as u16, 500);
        assert_eq!(HttpStatus::ImATeapot as u16, 418);
    }
}
