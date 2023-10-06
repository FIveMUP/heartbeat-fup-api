use crate::error::{AppError, AppResult};

pub(crate) async fn index() -> AppResult<&'static str> {
    if true {
        return Err(AppError::InternalServerError);
    }

    Ok("Hello, world!")
}
