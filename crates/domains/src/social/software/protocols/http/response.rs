/// HTTP status code classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusClass {
    Informational, // 1xx
    Success,       // 2xx
    Redirection,   // 3xx
    ClientError,   // 4xx
    ServerError,   // 5xx
}

/// HTTP status code with enforcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusCode(pub u16);

impl StatusCode {
    pub const OK: Self = StatusCode(200);
    pub const CREATED: Self = StatusCode(201);
    pub const NO_CONTENT: Self = StatusCode(204);
    pub const MOVED: Self = StatusCode(301);
    pub const NOT_FOUND: Self = StatusCode(404);
    pub const SERVER_ERROR: Self = StatusCode(500);

    pub fn new(code: u16) -> Result<Self, &'static str> {
        if !(100..=599).contains(&code) {
            return Err("status code must be 100-599");
        }
        Ok(StatusCode(code))
    }

    pub fn class(&self) -> StatusClass {
        match self.0 {
            100..=199 => StatusClass::Informational,
            200..=299 => StatusClass::Success,
            300..=399 => StatusClass::Redirection,
            400..=499 => StatusClass::ClientError,
            500..=599 => StatusClass::ServerError,
            code => panic!("StatusCode({code}) outside valid 100-599 range"),
        }
    }

    pub fn is_success(&self) -> bool {
        self.class() == StatusClass::Success
    }
    pub fn is_error(&self) -> bool {
        matches!(
            self.class(),
            StatusClass::ClientError | StatusClass::ServerError
        )
    }
    pub fn is_redirect(&self) -> bool {
        self.class() == StatusClass::Redirection
    }
}

/// HTTP response.
#[derive(Debug, Clone)]
pub struct Response {
    pub status: StatusCode,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

impl Response {
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: None,
        }
    }

    /// HEAD responses must not have a body.
    pub fn for_head(status: StatusCode) -> Result<Self, &'static str> {
        Ok(Self {
            status,
            headers: Vec::new(),
            body: None,
        })
    }

    /// 204 No Content must not have a body.
    pub fn no_content() -> Self {
        Self {
            status: StatusCode::NO_CONTENT,
            headers: Vec::new(),
            body: None,
        }
    }
}
