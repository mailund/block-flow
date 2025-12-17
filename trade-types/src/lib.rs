use serialization_macros::{serializable_enum, serializable_struct};

#[serializable_struct]
pub struct Contract(String);

impl Contract {
    pub fn new(name: &str) -> Self {
        Contract(name.to_string())
    }
}

#[serializable_struct]
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

#[serializable_struct]
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
impl From<Kw> for Quantity {
    fn from(k: Kw) -> Self {
        Quantity { kw: k.0 }
    }
}

pub struct Mw(pub u32);
impl From<Mw> for Quantity {
    fn from(m: Mw) -> Self {
        Quantity { kw: m.0 * 1_000 }
    }
}

#[serializable_enum]
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
