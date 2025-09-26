use pingora::http::RequestHeader;
use pingora::prelude::*;

pub fn add_forwarded_headers(request: &mut RequestHeader) -> Result<()> {
    request.insert_header("X-Forwarded-By", "langspec-gateway")?;
    Ok(())
}
