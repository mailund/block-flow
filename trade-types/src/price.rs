use super::*;

#[serializable_struct]
#[derive(PartialEq, Eq)]
pub struct Price {
    cents: u32,
}
impl Price {
    pub fn in_cents(&self) -> Cents {
        Cents(self.cents)
    }
    pub fn in_euros(&self) -> Euros {
        Euros(self.cents / 100)
    }
}

pub struct Cents(pub u32);
impl From<Cents> for Price {
    fn from(c: Cents) -> Self {
        Price { cents: c.0 }
    }
}

pub struct Euros(pub u32);
impl From<Euros> for Price {
    fn from(e: Euros) -> Self {
        Price { cents: e.0 * 100 }
    }
}
