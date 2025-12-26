use super::*;

#[derive(PartialEq, Eq, Hash, Clone, Debug, serde::Serialize, serde::Deserialize, Serializable)]
pub struct Contract(String);

impl Contract {
    pub fn new(name: &str) -> Self {
        Contract(name.to_string())
    }
}
