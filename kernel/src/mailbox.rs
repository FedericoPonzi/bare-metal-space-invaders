use crate::framebuffer::FrameBuffer;
use crate::mailbox::ReqResp::ResponseSuccessful;
use crate::mmio::VIDEOCORE_MBOX_BASE;
use crate::{debug, error};

use core::mem;
use core::ops::BitAnd;
use cortex_a::asm;
use log::info;
use space_invaders::{SCREEN_HEIGHT, SCREEN_WIDTH};
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::registers::{ReadOnly, WriteOnly};

const LFB_MESSAGE_SIZE: usize = 35;
/// Set physical (display) width/height
const FB_PHYSICAL_WH_TAG: u32 = 0x00048003;
/// Width of the requested frame buffer
const FB_PHYSICAL_WIDTH: u32 = SCREEN_WIDTH;
/// Height of the requested frame buffer
const FB_PHYSICAL_HEIGHT: u32 = SCREEN_HEIGHT;

pub const FB_BUFFER_LEN: usize = FB_PHYSICAL_HEIGHT as usize * FB_PHYSICAL_WIDTH as usize;

/// Set virtual (buffer) width/height
const FB_VIRTUAL_WH_TAG: u32 = 0x00048004;
const FB_VIRTUAL_WIDTH: u32 = SCREEN_WIDTH;
const FB_VIRTUAL_HEIGHT: u32 = SCREEN_HEIGHT * 2;

pub const TOTAL_FB_BUFFER_LEN: usize = FB_VIRTUAL_HEIGHT as usize * FB_VIRTUAL_WIDTH as usize;

const FB_VIRTUAL_OFFSET_TAG: u32 = 0x48009;
const FB_VIRTUAL_OFFSET_X: u32 = 0;
const FB_VIRTUAL_OFFSET_Y: u32 = 0;

// TODO: wrap into registers map lib
#[repr(C)]
struct RawMailbox {
    read: ReadOnly<u32>,
    _unused: u32,
    _unused2: u32,
    _unused3: u32,
    poll: u32,
    sender: u32,
    status: ReadOnly<u32>,
    config: u32,
    write: WriteOnly<u32>,
}
//const_assert_size!(RawMailbox, 36);

#[inline(always)]
pub fn nop() {
    asm::nop();
}

impl RawMailbox {
    pub(crate) fn is_empty(&self) -> bool {
        let status = self.get_status();
        status & STATUS_EMPTY == STATUS_EMPTY
    }

    fn is_full(&self) -> bool {
        let status = self.get_status();
        status & STATUS_FULL == STATUS_FULL
    }

    pub(crate) fn get_read(&self) -> u32 {
        self.read.get()
    }

    pub(crate) fn write_address(&mut self, address: usize) {
        self.write.set(address as u32)
    }

    fn get_status(&self) -> u32 {
        self.status.get()
    }
}

const STATUS_FULL: u32 = 0x80000000;
const STATUS_EMPTY: u32 = 0x40000000;

impl RawMailbox {}

#[derive(Debug, Copy, Clone)]
enum ReqResp {
    ResponseSuccessful,
    ResponseError,
    Request,
}

impl PartialEq<Self> for ReqResp {
    fn eq(&self, other: &Self) -> bool {
        let other = *other as u32;
        (*self as u32).eq(&other)
    }
}

impl Eq for ReqResp {}

impl Into<u32> for ReqResp {
    fn into(self) -> u32 {
        use ReqResp::*;
        match self {
            Request => 0x00000000,
            ResponseSuccessful => 0x80000000,
            ResponseError => 0x80000001,
        }
    }
}
impl From<u32> for ReqResp {
    fn from(val: u32) -> Self {
        use ReqResp::*;
        match val {
            0x00000000 => Request,
            0x80000000 => ResponseSuccessful,
            _ => ResponseError,
        }
    }
}
const MBOX_REQUEST: u32 = 0;
const BOARD_SERIAL_REQ: u32 = 0x00010004;
const GET_MAX_CLOCK_RATE: u32 = 0x00030004;
const SET_CLOCK_RATE: u32 = 0x00038002;
const SET_VIRTUAL_BUFFER_OFFSET_TAG: u32 = 0x00048009;
const TEST_SET_VIRTUAL_BUFFER_OFFSET_TAG: u32 = 0x00044009;
const LAST_TAG: u32 = 0;

#[repr(align(16))]
#[derive(Debug, Copy, Clone)]
struct Message<const T: usize>([u32; T]);

impl<const T: usize> Message<T> {
    pub fn response_status(&self) -> ReqResp {
        ReqResp::from(self.0[1])
    }
    pub fn is_response_successfull(&self) -> bool {
        self.response_status() == ResponseSuccessful
    }
}

pub fn query_board_serial() -> Option<u64> {
    debug!("Preparing board message..");
    let message = board_serial_message();
    debug!("Sending message to channel PROP: {:?}", message);

    return if send_message_sync(Channel::PROP, &message) {
        info!(
            "Serial number is: {:#04x}/{:#04x}",
            message.0[5], message.0[4]
        );
        let b = message.0[4].to_ne_bytes();
        let c = message.0[5].to_ne_bytes();
        let single = [b[0], b[1], b[2], b[3], c[0], c[1], c[2], c[3]];
        info!("Single: {:?}", single);
        Some(u64::from_ne_bytes(single))
    } else {
        info!("Failed to sending message to query the board serial.");
        None
    };
}

const fn lfb_message() -> Message<LFB_MESSAGE_SIZE> {
    let mut ret = [0u32; LFB_MESSAGE_SIZE];
    ret[0] = (LFB_MESSAGE_SIZE * mem::size_of::<u32>()) as u32;
    ret[1] = MBOX_REQUEST;
    //set phy width:
    ret[2] = FB_PHYSICAL_WH_TAG;
    ret[3] = 8;
    ret[4] = 8;
    //FrameBufferInfo.width
    ret[5] = FB_PHYSICAL_WIDTH;
    //FrameBufferInfo.height
    ret[6] = FB_PHYSICAL_HEIGHT;

    //set virt wh
    ret[7] = FB_VIRTUAL_WH_TAG;
    ret[8] = 8;
    ret[9] = 8;
    //FrameBufferInfo.virtual_width
    ret[10] = FB_VIRTUAL_WIDTH;
    //FrameBufferInfo.virtual_height
    ret[11] = FB_VIRTUAL_HEIGHT;

    // set virt offset
    ret[12] = FB_VIRTUAL_OFFSET_TAG;
    ret[13] = 8;
    ret[14] = 8;
    ret[15] = FB_VIRTUAL_OFFSET_X;
    ret[16] = FB_VIRTUAL_OFFSET_Y;

    ret[17] = 0x48005; //set depth
    ret[18] = 4;
    ret[19] = 4;
    ret[20] = 32; //FrameBufferInfo.depth

    ret[21] = 0x48006; //set pixel order
    ret[22] = 4;
    ret[23] = 4;
    ret[24] = 1; //RGB, not BGR preferably

    ret[25] = 0x40001; // Allocate buffer
    ret[26] = 8;
    ret[27] = 8;
    ret[28] = 4096; //FrameBufferInfo.pointer
    ret[29] = 0; //FrameBufferInfo.size

    ret[30] = 0x40008; //get pitch
    ret[31] = 4;
    ret[32] = 4;
    ret[33] = 0; //FrameBufferInfo.pitch

    ret[34] = LAST_TAG;
    Message(ret)
}

pub fn lfb_init<'a: 'static>(tentative: usize) -> Option<FrameBuffer> {
    let message = lfb_message();
    let res = send_message_sync(Channel::PROP, &message);
    return if res && message.0[28] != 0 {
        //convert GPU address to ARM address
        let fb_ptr_raw = (message.0[28] & 0x3FFFFFFF) as usize;

        //get actual physical width
        let width = message.0[5];
        //get actual physical height
        let height = message.0[6];
        // get number of bytes per line:
        let pitch = message.0[33];
        // get the pixel depth TODO: is this correct? Missin from: https://github.com/bztsrc/raspi3-tutorial/blob/master/09_framebuffer/lfb.c
        let depth = message.0[20];
        //get the actual channel order. brg = 0, rgb > 0
        let is_rgb = message.0[24] != 0;

        let casted = fb_ptr_raw as *const u32 as *mut u32;
        let casted = unsafe { &mut *casted };
        let framebuff: &mut [u32] =
            unsafe { core::slice::from_raw_parts_mut(casted, TOTAL_FB_BUFFER_LEN) };
        let fb = FrameBuffer {
            framebuff,
            width,
            height,
            pitch,
            depth_bits: depth,
            is_rgb,
            is_brg: !is_rgb,
            fb_virtual_width: FB_VIRTUAL_WIDTH,
            current_index: 0,
        };
        info!(
            "All good, setting up the frame buffer now: {}, height: {}, pitch: {}, depth:{}, is_rgb: {}",
            width, height, pitch, depth, is_rgb
        );
        Some(fb)
    } else {
        error!(
            "Something went wrong setting up lfb. Send message: {}, lfb address: {}",
            res, message.0[28]
        );
        if tentative == 1 {
            None
        } else {
            error!("trying again");
            lfb_init(1)
        }
    };
}

pub fn set_clock_speed(new_clock: u32) {
    let message = get_set_clock_rate_message(new_clock);
    info!(
        "Sending message to channel PROP to set clock speed: {:?}",
        message
    );

    if send_message_sync(Channel::PROP, &message) {
        let clock_id = message.0[4];
        let rate = message.0[5];

        info!("New rate for {:?} is: {:?}", clock_id, rate);
    } else {
        info!("Failed to sending message to set clock speed.");
    }
}

pub fn set_virtual_framebuffer_offset(offset: u32) {
    let message = get_set_virtual_framebuffer_offset_message(offset);

    if send_message_sync(Channel::PROP, &message) {
        //let offset_x = message.0[5];
        //let offset_y = message.0[6];
        //info!("New offset: {}, y{}", offset_x, offset_y);
    } else {
        error!("Failed to sending message to set virtual framebuffer offset.");
    }
}

pub fn test_set_virtual_framebuffer_offset(offset: u32) {
    let message = get_test_virtual_fb_offset_message(offset);

    if send_message_sync(Channel::PROP, &message) {
        let offset_x = message.0[5];
        let offset_y = message.0[6];
        info!(
            " requested offset: {} new offset: {}, y{}",
            offset, offset_x, offset_y
        );
    } else {
        error!("Failed to sending message to set virtual framebuffer offset.");
    }
}

pub fn max_clock_speed() -> Option<u32> {
    //command 0x00030004 ARM clock ID = 0x3
    // BCM2835_MAILBOX_TAG_GET_MAX_CLOCK_RATE 0x00030004

    let message = max_clock_rate_message();
    info!(
        "Sending message to channel PROP for max clock speed: {:?}",
        message
    );

    if send_message_sync(Channel::PROP, &message) {
        info!("message: {:?}", message);
        let core_id = message.0[5];
        let max_speed_hz = message.0[6];
        info!(
            "Max clock speed for core id: {:?} is : {:?}hz",
            core_id, max_speed_hz
        );
        Some(max_speed_hz)
    } else {
        info!("Failed to sending message to query max clock speed.");
        None
    }
}

const SET_VIRTUAL_FRAMEBUFFER_OFFSET_MESSAGE_SIZE: usize = 8;
fn get_set_virtual_framebuffer_offset_message(
    offset_y: u32,
) -> Message<SET_VIRTUAL_FRAMEBUFFER_OFFSET_MESSAGE_SIZE> {
    let mut ret = [0u32; SET_VIRTUAL_FRAMEBUFFER_OFFSET_MESSAGE_SIZE];
    ret[0] = (SET_VIRTUAL_FRAMEBUFFER_OFFSET_MESSAGE_SIZE * mem::size_of::<u32>()) as u32;
    ret[1] = MBOX_REQUEST;
    ret[2] = SET_VIRTUAL_BUFFER_OFFSET_TAG; // set virtual buffer offset
    ret[3] = 2 * mem::size_of::<u32>() as u32; // value buffer size in bytes
    ret[4] = 0; // :b 31 clear: request, | b31 set: response b30-b0: value length in bytes
    ret[5] = 0; //x in pixels
    ret[6] = offset_y; // y in pixels
    ret[7] = LAST_TAG;
    Message(ret)
}

const TEST_SET_VIRTUAL_FRAMEBUFFER_OFFSET_MESSAGE_SIZE: usize = 8;
fn get_test_virtual_fb_offset_message(
    offset_y: u32,
) -> Message<TEST_SET_VIRTUAL_FRAMEBUFFER_OFFSET_MESSAGE_SIZE> {
    let mut ret = [0u32; TEST_SET_VIRTUAL_FRAMEBUFFER_OFFSET_MESSAGE_SIZE];
    ret[0] = (TEST_SET_VIRTUAL_FRAMEBUFFER_OFFSET_MESSAGE_SIZE * mem::size_of::<u32>()) as u32;
    ret[1] = MBOX_REQUEST;
    ret[2] = SET_VIRTUAL_BUFFER_OFFSET_TAG; // set virtual buffer offset
    ret[3] = 2 * mem::size_of::<u32>() as u32; // value buffer size in bytes
    ret[4] = 0; // :b 31 clear: request, | b31 set: response b30-b0: value length in bytes
    ret[5] = 0; //x in pixels
    ret[6] = offset_y; // y in pixels
    ret[7] = LAST_TAG;
    Message(ret)
}

const GET_CLOCK_RATE_MESSAGE_SIZE: usize = 10;
fn get_set_clock_rate_message(new_clock_hz: u32) -> Message<GET_CLOCK_RATE_MESSAGE_SIZE> {
    let mut ret = [0u32; GET_CLOCK_RATE_MESSAGE_SIZE];
    ret[0] = (GET_CLOCK_RATE_MESSAGE_SIZE * mem::size_of::<u32>()) as u32;
    ret[1] = MBOX_REQUEST;

    ret[2] = SET_CLOCK_RATE; // set clock rate
    ret[3] = 3 * mem::size_of::<u32>() as u32; // value buffer size in bytes
    ret[4] = 0x1; //clock id
    ret[5] = new_clock_hz; // rate in hz
    ret[6] = 0; // skip setting turbo
    ret[7] = 0;
    ret[8] = LAST_TAG;
    Message(ret)
}

/// rate in hz.
const MAX_CLOCK_RATE_MESSAGE_SIZE: usize = 9;
fn max_clock_rate_message() -> Message<MAX_CLOCK_RATE_MESSAGE_SIZE> {
    let mut ret = [0u32; MAX_CLOCK_RATE_MESSAGE_SIZE];
    ret[0] = (MAX_CLOCK_RATE_MESSAGE_SIZE * mem::size_of::<u32>()) as u32;
    ret[1] = MBOX_REQUEST;

    //tag:
    ret[2] = GET_MAX_CLOCK_RATE; // get serial number command
    ret[3] = 8; // value buffer size in bytes
    ret[4] = 8; // :b 31 clear: request, | b31 set: response b30-b0: value length in bytes

    ret[5] = 0x1; //clock id
    ret[6] = 0; // used by the response.
    ret[7] = LAST_TAG;
    Message(ret)
}

const SERIAL_MESSAGE_SIZE: usize = 9;
fn board_serial_message() -> Message<SERIAL_MESSAGE_SIZE> {
    const SERIAL_MESSAGE_TAG: u32 = 0x00010004;
    let mut ret = [0u32; SERIAL_MESSAGE_SIZE];
    ret[0] = (SERIAL_MESSAGE_SIZE * mem::size_of::<u32>()) as u32;
    ret[1] = MBOX_REQUEST;

    ret[2] = SERIAL_MESSAGE_TAG; // tag identifier
    ret[3] = 8; // value buffer size in bytes
    ret[4] = 8; //Request codes:b 31 clear: request
    ret[5] = 8; // clear output buffer
    ret[6] = 0;

    ret[7] = LAST_TAG;
    Message(ret)
}

fn send_message_sync<const T: usize>(channel: Channel, message: &Message<T>) -> bool {
    let raw_ptr = message.0.as_ptr();
    // This is needed because slices are fat pointers and I need to convert it to a thin pointer first.
    let raw_ptr_addr = raw_ptr.cast::<usize>();
    let raw_ptr_addr = raw_ptr_addr as usize;
    // !0x0F is 1...10000
    let addr_clear_last_4_bits = raw_ptr_addr.bitand(!0x0F);
    let ch_clear_everything_but_last_4_vits = channel as usize & 0xF;
    let final_addr = addr_clear_last_4_bits | ch_clear_everything_but_last_4_vits;

    let raw_mailbox_ptr = VIDEOCORE_MBOX_BASE as *mut RawMailbox;
    let raw_mailbox = unsafe { &mut *raw_mailbox_ptr };

    /* wait until we can write to the mailbox */
    while raw_mailbox.is_full() {
        nop();
        nop();
        nop();
        nop();
    }

    raw_mailbox.write_address(final_addr);

    /* now wait for the response */
    loop {
        /* is there a response? */
        while raw_mailbox.is_empty() {
            nop();
            nop();
            nop();
            nop();
            nop();
        }

        if raw_mailbox.get_read() == final_addr as u32 {
            return match message.response_status() {
                ReqResp::Request => {
                    debug!("message stll contains a request ?!");
                    false
                }
                ReqResp::ResponseError => {
                    error!("Something failed, the response is an error");
                    false
                }
                ReqResp::ResponseSuccessful => true,
            };
        }
    }
}
#[derive(Copy, Clone)]
pub enum Channel {
    POWER = 0,
    FB = 1,
    VUART = 2,
    VCHIQ = 3,
    LEDS = 4,
    BTNS = 5,
    TOUCH = 6,
    COUNT = 7,
    PROP = 8,
}
