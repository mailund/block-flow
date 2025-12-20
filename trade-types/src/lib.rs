use serialization_macros::{serializable_enum, serializable_struct};

#[serializable_struct]
#[derive(PartialEq, Eq, Hash)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_new_and_equality() {
        let a = Contract::new("TEST");
        let b = Contract::new("TEST");
        let c = Contract::new("OTHER");

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn price_from_cents_roundtrip_in_cents_and_euros() {
        let p: Price = Cents(12345).into();
        assert_eq!(p.in_cents().0, 12345);
        assert_eq!(p.in_euros().0, 123); // integer division
    }

    #[test]
    fn price_from_euros_converts_to_cents_correctly() {
        let p: Price = Euros(42).into();
        assert_eq!(p.in_cents().0, 4200);
        assert_eq!(p.in_euros().0, 42);
    }

    #[test]
    fn quantity_from_kw_roundtrip_in_kw_and_mw() {
        let q: Quantity = Kw(2500).into();
        assert_eq!(q.in_kw().0, 2500);
        assert_eq!(q.in_mw().0, 2); // integer division
    }

    #[test]
    fn quantity_from_mw_converts_to_kw_correctly() {
        let q: Quantity = Mw(3).into();
        assert_eq!(q.in_kw().0, 3000);
        assert_eq!(q.in_mw().0, 3);
    }

    #[test]
    fn side_enum_variants_exist_and_match() {
        let b = Side::Buy;
        let s = Side::Sell;

        match b {
            Side::Buy => {}
            Side::Sell => panic!("expected Buy"),
        }

        match s {
            Side::Sell => {}
            Side::Buy => panic!("expected Sell"),
        }
    }

    #[test]
    fn orderbook_top_of_side_returns_some_for_both_sides() {
        let ob = Orderbook;

        let buy = ob.top_of_side(Side::Buy);
        let sell = ob.top_of_side(Side::Sell);

        assert!(buy.is_some());
        assert!(sell.is_some());

        assert_eq!(buy.unwrap(), 100.0);
        assert_eq!(sell.unwrap(), 100.0);
    }
}
