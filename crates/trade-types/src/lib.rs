use serialization_macros::Serializable;

mod contract;
pub use contract::Contract;

mod price;
pub use price::{Cents, Euros, Price};

mod quantity;
pub use quantity::{Kw, Mw, Quantity};

#[derive(PartialEq, Eq, Hash, Debug, Clone, serde::Serialize, serde::Deserialize, Serializable)]
pub enum Side {
    Buy,
    Sell,
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
}
