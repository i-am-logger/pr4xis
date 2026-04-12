use pr4xis::category::Entity;

/// HTTP methods with their semantic properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl Method {
    /// Is this method safe (no side effects)?
    pub fn is_safe(&self) -> bool {
        matches!(self, Method::Get | Method::Head | Method::Options)
    }

    /// Is this method idempotent (same result if repeated)?
    pub fn is_idempotent(&self) -> bool {
        matches!(
            self,
            Method::Get | Method::Put | Method::Delete | Method::Head | Method::Options
        )
    }

    /// Does this method typically have a request body?
    pub fn has_body(&self) -> bool {
        matches!(self, Method::Post | Method::Put | Method::Patch)
    }

    pub fn all() -> Vec<Method> {
        vec![
            Method::Get,
            Method::Post,
            Method::Put,
            Method::Delete,
            Method::Patch,
            Method::Head,
            Method::Options,
        ]
    }
}

/// An HTTP request with enforcement.
#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

impl Request {
    /// Create a request. Enforces: safe methods can't have body.
    pub fn new(method: Method, path: &str) -> Self {
        Self {
            method,
            path: path.to_string(),
            headers: Vec::new(),
            body: None,
        }
    }

    /// Set body. Returns Err if method doesn't support body.
    pub fn with_body(mut self, body: Vec<u8>) -> Result<Self, &'static str> {
        if !self.method.has_body() {
            return Err("method does not support request body");
        }
        self.body = Some(body);
        Ok(self)
    }

    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }
}
