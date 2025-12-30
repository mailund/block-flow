use std::fmt::Debug;

use trade_types::*;

mod actor;
mod actor_execution_context;
mod controller;
mod orders;
pub use actor::{Actor, ActorAlgo, ActorTrait};
pub use actor_execution_context::ActorExecutionContext;
pub use controller::ActorController;
pub use orders::Order;

/// Mock delta
#[derive(Debug)]
pub struct Delta(pub Contract);
