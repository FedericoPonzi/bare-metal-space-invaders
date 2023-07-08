#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Coordinates {
    pub(crate) x: u32,
    pub(crate) y: u32,
}

impl Coordinates {}

impl Coordinates {
    #[inline(always)]
    pub(crate) fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    #[inline(always)]
    pub(crate) fn add_x(&mut self, x: u32) {
        self.x += x;
    }

    #[inline(always)]
    pub(crate) fn sub_x(&mut self, x: u32) {
        self.x = self.x.saturating_sub(x);
    }

    pub(crate) fn add_y(&mut self, p0: u32) {
        self.y = self.y.saturating_add(p0);
    }

    pub(crate) fn sub_y(&mut self, p0: u32) {
        self.y = self.y.saturating_sub(p0);
    }
}
