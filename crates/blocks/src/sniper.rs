use super::*;
use block_traits::{
    execution_context::OrderBookTrait,
    intents::{Intent, IntentFactory, OneIntent},
    BlockSpec, ExecutionContextTrait,
};
use trade_types::*;

make_defaults!(state, output);

#[input]
pub struct Input {
    pub should_execute: bool,
}

#[init_params]
pub struct InitParams {
    pub contract: Contract,
    pub side: Side,
    pub quantity: Quantity,
    pub threshold: Price,
}

#[block(intents = OneIntent)]
pub struct SniperBlock {
    block_id: u32,
    contract: Contract,
    side: Side,
    quantity: Quantity,
    threshold: Price,
}

impl SniperBlock {
    fn place_intent(&self, price: Price) -> Intent {
        Intent::place_intent(
            self.contract.clone(),
            self.side.clone(),
            price.clone(),
            self.quantity.clone(),
        )
    }

    fn snipe_buy<OB: OrderBookTrait>(&self, order_book: &OB) -> Intent {
        if let Some(top_price) = order_book.top_of_side(Side::Sell) {
            if top_price <= self.threshold {
                return self.place_intent(top_price);
            }
        }
        Intent::no_intent()
    }

    fn snipe_sell<OB: OrderBookTrait>(&self, order_book: &OB) -> Intent {
        if let Some(top_price) = order_book.top_of_side(Side::Buy) {
            if top_price >= self.threshold {
                return self.place_intent(top_price);
            }
        }
        Intent::no_intent()
    }

    fn intents<C: ExecutionContextTrait>(&self, ctx: &C, execute: bool) -> Option<OneIntent> {
        let order_book = ctx.get_order_book(&self.contract)?;
        let intent = match (execute, &self.side) {
            (true, Side::Buy) => self.snipe_buy(&order_book),
            (true, Side::Sell) => self.snipe_sell(&order_book),
            _ => Intent::no_intent(),
        };
        Some(OneIntent::new([intent]))
    }
}

impl BlockSpec for SniperBlock {
    fn block_id(&self) -> u32 {
        self.block_id
    }

    fn new_from_init_params(
        InitParams {
            contract,
            side,
            quantity,
            threshold,
        }: &InitParams,
    ) -> Self {
        SniperBlock {
            block_id: 0,
            contract: contract.clone(),
            side: side.clone(),
            quantity: quantity.clone(),
            threshold: threshold.clone(),
        }
    }

    fn init_state(&self) -> State {
        State
    }

    #[execute]
    fn execute<C: ExecutionContextTrait>(
        &self,
        ctx: &C,
        Input { should_execute }: Input,
    ) -> Option<Self::Intents> {
        self.intents(ctx, should_execute)
    }
}
