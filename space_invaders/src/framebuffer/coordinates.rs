#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Coordinates {
    pub virtual_x: f64,
    pub virtual_y: f64,
}

impl Coordinates {
    #[inline(always)]
    pub fn new(x: u32, y: u32) -> Self {
        Self {
            virtual_x: x as f64,
            virtual_y: y as f64,
        }
    }
    #[inline(always)]
    pub fn x(&self) -> u32 {
        self.virtual_x as u32
    }
    #[inline(always)]
    pub fn y(&self) -> u32 {
        self.virtual_y as u32
    }

    #[inline(always)]
    pub fn add_virtual_x(&mut self, x: f64) {
        self.virtual_x += x;
    }
    #[inline(always)]
    pub fn sub_virtual_x(&mut self, x: f64) {
        self.virtual_x -= x;
    }

    #[inline(always)]
    pub fn set_virtual_x(&mut self, x: f64) {
        self.virtual_x = x;
    }
    #[inline(always)]
    pub fn sub_virtual_y(&mut self, speed: f64, delta: u64) {
        self.virtual_y -= speed * delta as f64;
    }
    #[inline(always)]
    pub fn add_virtual_y(&mut self, speed: f64, delta: u64) {
        self.virtual_y += speed * delta as f64;
    }
}
