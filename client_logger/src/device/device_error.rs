use core::fmt::{self, Display};

#[derive(Debug)]
pub struct DeviceError {
    msg: String,
}

impl DeviceError {
    pub fn new(msg: &str) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }
}

impl Display for DeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for DeviceError {}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::quickcheck;

    quickcheck! {
        fn prop_display_preserves_error_message(msg: String) -> bool {
            let error = DeviceError::new(&msg);
            format!("{}", error) == msg
        }
    }
}
