use core::fmt::{self, Display};

#[derive(Debug)]
pub struct WeatherError {
    msg: String,
}

impl WeatherError {
    pub fn new(msg: &str) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }
}

impl Display for WeatherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for WeatherError {}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::quickcheck;

    quickcheck! {
        fn prop_display_preserves_error_message(msg: String) -> bool {
            let error = WeatherError::new(&msg);
            format!("{}", error) == msg
        }
    }
}
