use std::result::Result as StdResult;

use crate::error::Error;

pub type Result<T> = StdResult<T, Error>;
