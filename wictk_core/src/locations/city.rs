use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct City {
    /// Location name to search for (e.g., "Oslo", "London")
    pub location: String,
}

impl TryFrom<Value> for City {
    type Error = serde_json::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}
