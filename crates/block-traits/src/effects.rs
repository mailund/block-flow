pub enum Effect {
    Suspend,
    Terminate,
    Timer(u64),
}

impl Effect {
    pub fn suspend() -> Self {
        Effect::Suspend
    }

    pub fn terminate() -> Self {
        Effect::Terminate
    }

    pub fn timer(duration: u64) -> Self {
        Effect::Timer(duration)
    }
}
