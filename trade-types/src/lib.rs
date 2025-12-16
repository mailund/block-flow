use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Contract(String);

impl Contract {
    pub fn new(name: &str) -> Self {
        Contract(name.to_string())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
impl Into<Price> for Cents {
    fn into(self) -> Price {
        Price { cents: self.0 }
    }
}

pub struct Euros(pub u32);
impl Into<Price> for Euros {
    fn into(self) -> Price {
        Price {
            cents: self.0 * 100,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Quantity {
    kw: u32,
}
impl Quantity {
    pub fn in_kw(&self) -> Kw {
        Kw(self.kw)
    }
    pub fn in_mw(&self) -> Mw {
        Mw(self.kw / 1000)
    }
}

pub struct Kw(pub u32);
impl Into<Quantity> for Kw {
    fn into(self) -> Quantity {
        Quantity { kw: self.0 }
    }
}

pub struct Mw(pub u32);
impl Into<Quantity> for Mw {
    fn into(self) -> Quantity {
        Quantity { kw: self.0 * 1000 }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Side {
    Buy,
    Sell,
}
pub struct Orderbook;

impl Orderbook {
    pub fn top_of_side(&self, _side: Side) -> Option<f64> {
        // Dummy implementation
        Some(100.0)
    }
}
