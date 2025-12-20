use super::*;

#[serializable_struct]
#[derive(PartialEq, Eq)]
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
