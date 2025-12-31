use trade_types::{Contract, Price, Quantity, Side};

#[derive(Clone, Debug, Default)]
pub enum Intent {
    #[default]
    NoIntent,
    Place {
        contract: Contract,
        side: Side,
        price: Price,
        quantity: Quantity,
    },
}
