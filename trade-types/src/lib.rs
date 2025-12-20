use serialization_macros::{serializable_enum, serializable_struct};

mod contract;
pub use contract::Contract;

mod price;
pub use price::{Cents, Euros, Price};

mod quantity;
pub use quantity::{Kw, Mw, Quantity};

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
