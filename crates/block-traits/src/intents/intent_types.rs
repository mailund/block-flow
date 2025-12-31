use trade_types::{Contract, Price, Quantity, Side};

#[derive(Clone, Debug)]
pub enum Intent {
    NoIntent,
    Place {
        contract: Contract,
        side: Side,
        price: Price,
        quantity: Quantity,
    },
}

impl Intent {
    pub fn no_intent() -> Intent {
        Intent::NoIntent
    }

    pub fn place_intent(
        contract: Contract,
        side: Side,
        price: Price,
        quantity: Quantity,
    ) -> Intent {
        Intent::Place {
            contract,
            side,
            price,
            quantity,
        }
    }
}

pub trait IntentFactory {
    fn no_intent() -> Intent {
        Intent::no_intent()
    }
    fn place_intent(contract: Contract, side: Side, price: Price, quantity: Quantity) -> Intent {
        Intent::place_intent(contract, side, price, quantity)
    }
}

impl IntentFactory for Intent {}
impl<B: crate::BlockSpec> IntentFactory for B {}
// impl<X, C, I, E> IntentFactory for X
// where
//     X: crate::ExecuteTrait<C, I, E>,
//     C: crate::ExecutionContextTrait,
//     I: IntentConsumerTrait,
//     E: crate::EffectConsumerTrait,
// {
// }
