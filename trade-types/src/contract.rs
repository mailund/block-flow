use super::*;

#[serializable_struct]
#[derive(PartialEq, Eq, Hash)]
pub struct Contract(String);

impl Contract {
    pub fn new(name: &str) -> Self {
        Contract(name.to_string())
    }
}
