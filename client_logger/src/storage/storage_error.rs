use core::fmt::{self, Display};

#[derive(Debug)]
pub struct StorageError {
    msg: String,
}

impl StorageError {
    pub fn new(msg: &str) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }
}

impl Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for StorageError {}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::quickcheck;

    quickcheck! {
        fn prop_display_preserves_error_message(msg: String) -> bool {
            let error = StorageError::new(&msg);
            format!("{}", error) == msg
        }
    }
}
