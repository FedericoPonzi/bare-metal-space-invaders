use crate::actor::{Actor, ActorStructure};
use crate::{Color, Coordinates, FrameBufferInterface};
use std::{cmp, mem};

use crate::framebuffer::color;
use crate::framebuffer::fb_trait::LETTER_WIDTH;
use crate::{SCREEN_MARGIN, SCREEN_WIDTH};

pub const UI_MAX_SCORE_LEN: usize = "High Score: 9999 - Current Score: 9999".len();
// +1 because it doesn't take into account the last letter's space to the end of the screen
pub const UI_SCORE_X: u32 = SCREEN_WIDTH - (UI_MAX_SCORE_LEN as u32 + 1) * LETTER_WIDTH as u32;
pub const UI_SCORE_Y: u32 = SCREEN_MARGIN / 2;
pub const UI_SCORE_COORDINATES: Coordinates = Coordinates::new(UI_SCORE_X, UI_SCORE_Y);
pub const UI_SCORE_COLOR: Color = color::WHITE_COLOR;

pub struct ScoreCount {
    current_score: u32,
    high_score: u32,
    structure: ActorStructure,
    high_score_updated: u32,
    current_score_updated: u32,
}

impl ScoreCount {
    pub(crate) fn new(current_score: u32, high_score: u32) -> ScoreCount {
        ScoreCount {
            current_score,
            high_score,
            structure: ActorStructure {
                width: 0,
                alive: true,
                height: 0,
                coordinates: UI_SCORE_COORDINATES,
                sprite: None,
            },
            high_score_updated: high_score,
            current_score_updated: current_score,
        }
    }
    pub(crate) fn update(&mut self, enemies_dead: usize) {
        self.current_score_updated =
            self.current_score + u32::try_from(enemies_dead).expect("Conversion failed");
        self.high_score_updated = cmp::max(self.current_score_updated, self.high_score);
    }
}

impl Actor for ScoreCount {
    fn get_structure(&self) -> &ActorStructure {
        &self.structure
    }

    fn set_coordinates(&mut self, coordinates: Coordinates) {
        self.structure.coordinates = coordinates;
    }

    fn draw(&self, fb: &mut impl FrameBufferInterface) {
        let mut message_buf = [0u8; UI_MAX_SCORE_LEN * mem::size_of::<char>()];
        let text = format_to_buffer(
            &mut message_buf,
            self.high_score_updated,
            self.current_score_updated,
        )
        .expect("TODO: panic message");

        let mut x = self.structure.coordinates.x();
        let y = self.structure.coordinates.y();
        for c in text.chars() {
            // right distance after each character
            x += LETTER_WIDTH as u32;
            fb.write_char(c, Coordinates::new(x, y), UI_SCORE_COLOR);
        }
    }
}

// Function to write formatted data into a buffer
fn format_to_buffer(
    buffer: &mut [u8],
    high_score: u32,
    current_score: u32,
) -> Result<&str, core::fmt::Error> {
    use core::fmt::Write;
    let mut output = BufferWrite::new(buffer);
    write!(
        output,
        "High Score: {high_score} - Current Score: {current_score}"
    )?;

    // Convert the buffer slice into a &str
    let written_length = output.written_length();
    let formatted_str = core::str::from_utf8(&buffer[..written_length]).unwrap();
    Ok(formatted_str)
}

// A custom implementation of core::fmt::Write for writing into a buffer
struct BufferWrite<'a> {
    buffer: &'a mut [u8],
    position: usize,
}

impl<'a> BufferWrite<'a> {
    fn new(buffer: &'a mut [u8]) -> Self {
        BufferWrite {
            buffer,
            position: 0,
        }
    }

    // Get the total number of bytes written so far
    fn written_length(&self) -> usize {
        self.position
    }
}

// Implement the Write trait for BufferWrite
impl<'a> core::fmt::Write for BufferWrite<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remaining_space = self.buffer.len() - self.position;

        if bytes.len() <= remaining_space {
            self.buffer[self.position..self.position + bytes.len()].copy_from_slice(bytes);
            self.position += bytes.len();
            Ok(())
        } else {
            Err(core::fmt::Error)
        }
    }
}
