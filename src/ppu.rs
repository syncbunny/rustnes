use crate::events::*;
use crate::io::*;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use std::{thread, time};

const CLOCKS_PAR_LINE: u32 = 341;
const DRAWABLE_LINES: u32 = 240;
const SCAN_LINES: u32 = 262;
const BG_PALETTE_BASE: usize = 0x3F00;
const SPRITE_PALETTE_BASE: usize = 0x3F10;

/* Control Regster1 &H2000 */
const FLAG_NMI_ON_VB: u8 = 0x80;
const CR1_BG_PATTABLE_MASK: u8 = 0x10; // 0: 0x0000, 1:0x1000
const CR1_SP_PATTABLE_MASK: u8 = 0x08; // 0: 0x0000, 1:0x1000
const FLAG_ADDR_INC: u8 = 0x04; // 0: +1, 1: +32
const CR1_NAMETABLE_MASK: u8 = 0x03;

/* Control Register2 &H2001 */
const CR2_FLAG_ENABLE_SPRITE: u8 = 0x10;
const CR2_FLAG_ENABLE_BG: u8 = 0x08;

/* Status Register &H2002 */
const FLAG_VBLANK: u8 = 0x80;
const FLAG_SP_HIT: u8 = 0x40;
const SCANLINE_SPLITE_OVER: u8 = 0x20;
const IFLAG_VBLANK: u8 = 0x7F;
const IFLAG_SP_HIT: u8 = 0xBF;

/* Sprite attributes */
const SPRITE_ATTRIBUTE_BACK: u8 = 0x20;
const SPRITE_ATTRIBUTE_FLIP_H: u8 = 0x40;
const SPRITE_ATTRIBUTE_FLIP_V: u8 = 0x80;

const COLOR_TABLE: [u8; 0x40 * 3] = [
    /* 00 */ 0x6b, 0x6b, 0x6b, 0x00, 0x10, 0x84, 0x08, 0x00, 0x8c, 0x42, 0x00, 0x7b,
    /* 04 */ 0x63, 0x00, 0x5a, 0x6b, 0x00, 0x10, 0x60, 0x00, 0x00, 0x4f, 0x35, 0x00,
    /* 08 */ 0x31, 0x4e, 0x18, 0x00, 0x5a, 0x21, 0x21, 0x5a, 0x10, 0x08, 0x52, 0x42,
    /* 0c */ 0x00, 0x39, 0x73, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    /* 10 */ 0xa5, 0xa5, 0xa5, 0x00, 0x42, 0xc6, 0x42, 0x29, 0xce, 0x6b, 0x00, 0xbd,
    /* 14 */ 0x94, 0x29, 0x94, 0x9c, 0x10, 0x42, 0x9c, 0x39, 0x00, 0x84, 0x5e, 0x21,
    /* 18 */ 0x5f, 0x7b, 0x21, 0x2d, 0x8c, 0x29, 0x18, 0x8e, 0x10, 0x2e, 0x86, 0x63,
    /* 1c */ 0x29, 0x73, 0x9c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    /* 20 */ 0xef, 0xef, 0xef, 0x5a, 0x8c, 0xff, 0x7b, 0x6b, 0xff, 0xa5, 0x5a, 0xff,
    /* 24 */ 0xd6, 0x4a, 0xff, 0xe7, 0x63, 0x9c, 0xde, 0x7b, 0x52, 0xce, 0x9c, 0x29,
    /* 28 */ 0xad, 0xb5, 0x31, 0x7b, 0xce, 0x31, 0x5a, 0xce, 0x52, 0x4a, 0xc6, 0x94,
    /* 2c */ 0x4a, 0xb5, 0xce, 0x52, 0x52, 0x52, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    /* 30 */ 0xef, 0xef, 0xef, 0xad, 0xc6, 0xff, 0xbd, 0xbd, 0xff, 0xce, 0xb5, 0xff,
    /* 34 */ 0xe7, 0xb5, 0xff, 0xf9, 0xbb, 0xdf, 0xf7, 0xc6, 0xb5, 0xde, 0xc6, 0x9c,
    /* 38 */ 0xd6, 0xd6, 0x94, 0xc6, 0xe7, 0x9c, 0xb5, 0xe7, 0xad, 0xad, 0xe7, 0xc6,
    /* 3c */ 0xad, 0xde, 0xe7, 0xad, 0xad, 0xad, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

macro_rules! SET_VBLANK {
    ($sr: expr) => {
        $sr |= FLAG_VBLANK;
    };
}

macro_rules! CLEAR_VBLANK {
    ($sr: expr) => {
        $sr &= IFLAG_VBLANK;
    };
}

macro_rules! get_nametable {
    ($cr1: expr) => {
        $cr1 & CR1_NAMETABLE_MASK
    };
}

macro_rules! SET_SPRITE_HIT {
    ($sr: expr) => {
        $sr |= FLAG_SP_HIT;
    };
}

macro_rules! CLEAR_SPRITE_HIT {
    ($sr: expr) => {
        $sr &= IFLAG_SP_HIT;
    };
}

macro_rules! get_bg_pattern_table_addr {
    ($cr1: expr) => {
        if ($cr1 & CR1_BG_PATTABLE_MASK) == 0 {
            0x0000
        } else {
            0x1000
        }
    };
}

macro_rules! get_sprite_pattern_table_addr {
    ($cr1: expr) => {
        if ($cr1 & CR1_SP_PATTABLE_MASK) == 0 {
            0x0000
        } else {
            0x1000
        }
    };
}

pub enum Mirror {
    HORIZONTAL,
    VARTICAL,
}

pub struct PPU {
    cr1: u8, // Control Register 1
    cr2: u8, // Control Register 1
    sr: u8,  // Status Register
    scroll_y: u8,
    scroll_x: u8,

    line: u32,
    line_clock: u32,

    write_mode: u8, // 0 or 1
    write_addr: u16,
    sprite_write_addr: usize,
    read_buffer: u8,
    mem: Vec<u8>,
    sprite_mem: Vec<u8>,
    sprite_buffer: Vec<u8>,
    sprite_id_buffer: Vec<u8>,
    sprite_buffer_len: usize,
    mirror: Mirror,

    pattern_lut: Vec<u8>,
    attr_addr_lut: Vec<usize>,
    line_buffer: Vec<u8>,

    io: Arc<Mutex<IO>>,
    event_queue: Arc<Mutex<EventQueue>>,
    vbr: Arc<(Mutex<VBR>, Condvar)>,
    last_frame_time: Instant,
    nowait: bool,

    pub frames: u32,
}

impl PPU {
    pub fn new(
        io: Arc<Mutex<IO>>,
        event_queue: Arc<Mutex<EventQueue>>,
        vbr: Arc<(Mutex<VBR>, Condvar)>,
    ) -> PPU {
        let mut ppu = PPU {
            cr1: 0,
            cr2: 0,
            sr: 0,
            scroll_x: 0,
            scroll_y: 0,

            line: 0,
            line_clock: 0,

            write_mode: 0,
            write_addr: 0,
            sprite_write_addr: 0,
            read_buffer: 0,
            mem: vec![0; 0x4000],
            sprite_mem: vec![0; 256],
            sprite_buffer: vec![0; 4 * 256],
            sprite_id_buffer: vec![0; 256],
            sprite_buffer_len: 0,
            mirror: Mirror::VARTICAL,

            pattern_lut: vec![0; 256 * 256 * 8], // Hi * Lo * x
            attr_addr_lut: vec![0; 32 * 32],
            line_buffer: vec![0; 256 * 4], // 256 x [R, G, B, Stencil]

            io: io,
            event_queue: event_queue,
            vbr: vbr,
            last_frame_time: Instant::now(),
            nowait: false,

            frames: 0,
        };
        ppu.generate_lut();

        return ppu;
    }

    pub fn reset(&mut self) {
        self.line = 0;
        self.line_clock = 0;
    }

    pub fn nowait(&mut self, b: bool) {
        self.nowait = b;
    }

    pub fn clock(&mut self) {
        if self.line == 0 && self.line_clock == 0 {
            self.frame_start();
        }
        if self.line_clock == 0 {
            //println!("PPU: CLEAR_SPRITE_HIT: {:02X}", self.sr);
            self.buffer_sprite(self.line);
            self.line_start();
        }

        self.render_bg(self.line_clock, self.line);
        if self.cr2 & CR2_FLAG_ENABLE_SPRITE != 0 {
            self.render_sprite(self.line_clock, self.line);
        }

        self.line_clock += 1;
        if self.line == 260 && self.line_clock == 1 {
            CLEAR_SPRITE_HIT!(self.sr);
        }
        if self.line_clock >= CLOCKS_PAR_LINE {
            //println!("PPU: line {}", self.line);
            self.line_end(self.line);
            self.line_clock = 0;
            self.line += 1;
            if self.line == DRAWABLE_LINES {
                self.start_VR();
            }
            if self.line >= SCAN_LINES {
                CLEAR_VBLANK!(self.sr);
                self.line = 0;
                self.frame_end();
            }
        }
    }

    pub fn set_mirror(&mut self, m: Mirror) {
        self.mirror = m;
    }

    // Mapping to 0x2000
    pub fn set_cr1(&mut self, n: u8) {
        //println!("PPU: set_cr1: {:02X}", n);
        self.cr1 = n;
    }

    // Mapping to 0x2001
    pub fn set_cr2(&mut self, n: u8) {
        //println!("PPU: set_cr2: {:02X}", n);
        self.cr2 = n;
    }

    // Mapping to 0x2003
    pub fn set_sprite_write_addr(&mut self, n: u8) {
        self.sprite_write_addr = n as usize;
    }

    // Mapping to 0x2004
    pub fn sprite_write(&mut self, n: u8) {
        self.sprite_mem[self.sprite_write_addr] = n;
        self.sprite_write_addr += 1;
        self.sprite_write_addr &= 0xFFusize;
    }

    pub fn get_sr(&mut self) -> u8 {
        let sr: u8 = self.sr;

        self.write_mode = 0;
        CLEAR_VBLANK!(self.sr);

        //println!("PPU: get_sr: {:02X}", sr);
        return sr;
    }

    pub fn set_scroll(&mut self, v: u8) {
        //println!("PPU: set_scroll: {:02X}", v);
        if self.write_mode == 0 {
            self.scroll_x = v;
            self.write_mode = 1;
        } else {
            self.scroll_y = v;
            self.write_mode = 0;
        }
    }

    pub fn set_write_addr(&mut self, v: u8) {
        if self.write_mode == 0 {
            self.write_addr &= 0x00FF;
            self.write_addr |= (v as u16) << 8;

            self.write_mode = 1;
        } else {
            self.write_addr &= 0xFF00;
            self.write_addr |= (v as u16);

            self.write_mode = 0;
        }
    }

    // Mapping to 0x2007
    pub fn write(&mut self, v: u8) {
        let addr = self.write_addr & 0x3FFF;
        let mut v = v;

        // background pallet or sprite pallet
        if addr >= 0x3F00 && addr <= 0x3FFF {
            v &= 0x3F;
        }

        self.mem[addr as usize] = v;

        // Increment write address
        if self.cr1 & FLAG_ADDR_INC == 0 {
            self.write_addr += 1;
        } else {
            self.write_addr += 32;
        }
    }

    // Mapping to 0x2007
    pub fn read(&mut self) -> u8 {
        let ret: u8;
        match self.write_addr {
            0..=0x3EFF => {
                ret = self.read_buffer;
                self.read_buffer = self.mem[self.write_addr as usize];
            }
            0x3F00..=0x3FFF => {
                ret = self.mem[self.write_addr as usize];
            }
            _ => {
                panic!("ppu.read: unmapped address: {:x}", self.write_addr);
            }
        }

        // Increment write address
        if self.cr1 & FLAG_ADDR_INC == 0 {
            self.write_addr += 1;
        } else {
            self.write_addr += 32;
        }

        return ret;
    }

    fn start_VR(&mut self) {
        SET_VBLANK!(self.sr);
        if (self.cr1 & FLAG_NMI_ON_VB) != 0 {
            let mut queue = self.event_queue.lock().unwrap();
            queue.push(Event::new(EventType::NMI));
        }

        //		let (vbr, cond) = &*self.vbr;
        //		let mut vbr = vbr.lock().unwrap();
        //		(*vbr).in_vbr = true;
        //		cond.notify_all();
    }

    fn line_start(&mut self) {
        let col = self.mem[SPRITE_PALETTE_BASE];
        let r = COLOR_TABLE[(col * 3 + 0) as usize];
        let g = COLOR_TABLE[(col * 3 + 1) as usize];
        let b = COLOR_TABLE[(col * 3 + 2) as usize];
        let s = 0;

        for x in 0..256 {
            self.line_buffer[x * 4 + 0] = r;
            self.line_buffer[x * 4 + 1] = g;
            self.line_buffer[x * 4 + 2] = b;
            self.line_buffer[x * 4 + 3] = s;
        }
    }

    fn line_end(&mut self, y: u32) {
        if y < DRAWABLE_LINES {
            self.io.lock().unwrap().draw_line(y, &self.line_buffer);
        }
    }

    fn frame_start(&mut self) {
        //println!("PPU: FrameStart");

        let col = self.mem[SPRITE_PALETTE_BASE];
        let r = COLOR_TABLE[(col * 3 + 0) as usize];
        let g = COLOR_TABLE[(col * 3 + 1) as usize];
        let b = COLOR_TABLE[(col * 3 + 2) as usize];

        //self.io.lock().unwrap().clear(r, g, b);
    }

    fn frame_end(&mut self) {
        let (vbr, cond) = &*self.vbr;
        let mut vbr = vbr.lock().unwrap();
        cond.wait(vbr).unwrap();

        let t = Instant::now();
        self.last_frame_time = t;

        self.frames += 1;
    }

    fn render_bg(&mut self, x: u32, y: u32) {
        if x >= 256 || y >= 240 {
            return;
        }

        // calc nametable id
        let nametable_id = get_nametable!(self.cr1);
        let mut scroll_x: u16 = self.scroll_x as u16;
        let mut scroll_y: u16 = self.scroll_y as u16;
        if nametable_id == 1 || nametable_id == 3 {
            scroll_x += 256;
        }
        if nametable_id == 2 || nametable_id == 3 {
            scroll_y += 240;
        }

        let xx: u32 = x as u32 + scroll_x as u32;
        let yy: u32 = y as u32 + scroll_y as u32;
        let xx = xx % 512;
        let yy = yy % 480;

        let mut nametable_id = if xx >= 256 { 1 } else { 0 };
        nametable_id = nametable_id | (if yy >= 240 { 2 } else { 0 });

        let mirror_h_nt_id: [u8; 4] = [0, 0, 2, 2];
        let mirror_v_nt_id: [u8; 4] = [0, 1, 0, 1];
        match self.mirror {
            Mirror::VARTICAL => {
                nametable_id = mirror_v_nt_id[nametable_id as usize];
            }
            Mirror::HORIZONTAL => {
                nametable_id = mirror_h_nt_id[nametable_id as usize];
            }
            _ => {}
        }

        // calc nametable address
        //  +-----------+-----------+
        //  | 2 ($2800) | 3 ($2C00) |
        //  +-----------+-----------+
        //  | 0 ($2000) | 1 ($2400) |
        //  +-----------+-----------+
        let nametable_base: [u32; 4] = [0x2000, 0x2400, 0x2800, 0x2C00];
        let u: u32 = (xx / 8) % 32; // [0 .. 32]
        let v: u32 = (yy / 8) % 30; // [0 .. 30]
        let addr: u32 = nametable_base[nametable_id as usize] + v * 32 + u;
        let pat_id: u8 = self.mem[addr as usize]; // pattern id [0..255]

        let pat_base: u16 = get_bg_pattern_table_addr!(self.cr1);
        let pat_addr: u16 = pat_base + ((pat_id as u16) << 4);
        let pat_addr_lo: u16 = pat_addr + (yy % 8) as u16;
        let pat_addr_hi: u16 = pat_addr_lo + 8;
        let pat_lo = self.mem[pat_addr_lo as usize];
        let pat_hi = self.mem[pat_addr_hi as usize];
        let pat = self.pattern_lut
            [(((pat_hi as usize) * 256 + (pat_lo as usize)) * 8 + (xx as usize) % 8) as usize];
        let attribute_table_base: [usize; 4] = [0x23C0, 0x27C0, 0x2BC0, 0x2FC0];
        let (r, g, b) = self.get_color(nametable_id, u, v, pat);

        if pat != 0 {
            let stencil = self.line_buffer[(x * 4 + 3) as usize];
            if stencil < 2 {
                self.line_buffer[(x * 4 + 0) as usize] = r;
                self.line_buffer[(x * 4 + 1) as usize] = g;
                self.line_buffer[(x * 4 + 2) as usize] = b;
                self.line_buffer[(x * 4 + 3) as usize] = 2;
            }
        }
    }

    fn buffer_sprite(&mut self, y: u32) {
        self.sprite_buffer_len = 0;
        for sprite_id in 0..64 {
            let sp_y = self.sprite_mem[(sprite_id * 4 + 0) as usize];
            let sp_n = self.sprite_mem[(sprite_id * 4 + 1) as usize];
            let sp_a = self.sprite_mem[(sprite_id * 4 + 2) as usize];
            let sp_x = self.sprite_mem[(sprite_id * 4 + 3) as usize];

            if y < (sp_y as u32) + 1 || y >= (sp_y as u32) + 1 + 8 {
                continue;
            }
            self.sprite_buffer[self.sprite_buffer_len * 4 + 0] = sp_y;
            self.sprite_buffer[self.sprite_buffer_len * 4 + 1] = sp_n;
            self.sprite_buffer[self.sprite_buffer_len * 4 + 2] = sp_a;
            self.sprite_buffer[self.sprite_buffer_len * 4 + 3] = sp_x;
            self.sprite_id_buffer[self.sprite_buffer_len] = sprite_id;

            self.sprite_buffer_len += 1;
            if (self.sprite_buffer_len == 7) {
                break;
            }
        }
    }

    fn render_sprite(&mut self, x: u32, y: u32) {
        if (y >= 239) {
            return;
        }
        if (x >= 255) {
            return;
        }

        for buffer_id in (0..self.sprite_buffer_len).rev() {
            let sprite_id: u8 = self.sprite_id_buffer[buffer_id];
            let sp_y = self.sprite_buffer[(buffer_id * 4 + 0) as usize];
            let sp_n = self.sprite_buffer[(buffer_id * 4 + 1) as usize];
            let sp_a = self.sprite_buffer[(buffer_id * 4 + 2) as usize];
            let sp_x = self.sprite_buffer[(buffer_id * 4 + 3) as usize];

            if x < sp_x as u32 || x >= (sp_x as u32) + 8 {
                continue;
            }

            let mut u = (x as u8).wrapping_sub(sp_x);
            if sp_a & SPRITE_ATTRIBUTE_FLIP_H != 0 {
                u = 7 - u;
            }
            let mut v = (y as u8).wrapping_sub(sp_y).wrapping_sub(1);
            if sp_a & SPRITE_ATTRIBUTE_FLIP_V != 0 {
                v = 7 - v;
            }
            let pat_base: u16 = get_sprite_pattern_table_addr!(self.cr1);
            let pat_addr: u16 = pat_base + ((sp_n as u16) << 4);
            let pat_addr_lo: u16 = pat_addr + v as u16;
            let pat_addr_hi: u16 = pat_addr_lo + 8;
            let pat_lo = self.mem[pat_addr_lo as usize];
            let pat_hi = self.mem[pat_addr_hi as usize];
            let pat = self.pattern_lut
                [(((pat_hi as usize) * 256 + (pat_lo as usize)) * 8 + (u as usize) % 8) as usize];
            if pat == 0 {
                continue;
            }
            let pat = pat | (sp_a & 0x03) << 2;
            let col = self.mem[SPRITE_PALETTE_BASE + pat as usize]; // [0..3F]
            let r = COLOR_TABLE[(col * 3 + 0) as usize];
            let g = COLOR_TABLE[(col * 3 + 1) as usize];
            let b = COLOR_TABLE[(col * 3 + 2) as usize];

            let stencil = self.line_buffer[(x * 4 + 3) as usize];
            if sp_a & SPRITE_ATTRIBUTE_BACK != 0 {
                if stencil < 1 {
                    self.line_buffer[(x * 4 + 0) as usize] = r;
                    self.line_buffer[(x * 4 + 1) as usize] = g;
                    self.line_buffer[(x * 4 + 2) as usize] = b;
                    self.line_buffer[(x * 4 + 3) as usize] = 1;
                }
            } else {
                self.line_buffer[(x * 4 + 0) as usize] = r;
                self.line_buffer[(x * 4 + 1) as usize] = g;
                self.line_buffer[(x * 4 + 2) as usize] = b;
                self.line_buffer[(x * 4 + 3) as usize] = 3;
            }

            let stencil = self.line_buffer[(x * 4 + 3) as usize];
            if sprite_id == 0 && (self.cr2 & CR2_FLAG_ENABLE_BG) != 0 && stencil != 0 {
                SET_SPRITE_HIT!(self.sr);
            }
        }
    }

    pub fn get_sprite_mem(&mut self) -> &mut Vec<u8> {
        &mut self.sprite_mem
    }

    pub fn set_crom(&mut self, crom: &[u8]) {
        let len: usize = if crom.len() < 0x2000 {
            crom.len()
        } else {
            0x2000
        };
        self.mem[0..len].copy_from_slice(crom);
    }

    fn get_color(&self, nametable_id: u8, u: u32, v: u32, pat: u8) -> (u8, u8, u8) {
        let attr_addr = self.attr_addr_lut[(v * 32 + u) as usize];
        let attribute_table_base: [usize; 4] = [0x23C0, 0x27C0, 0x2BC0, 0x2FC0];
        let attr_addr = attribute_table_base[nametable_id as usize] + attr_addr;
        let mut attr = self.mem[attr_addr];

        // 01|23
        // 45|67
        // --+--
        // 89|AB
        // CD|EF
        let n = (v % 4) * 4 + u % 4; // [0..16]
        match n {
            0 | 1 | 4 | 5 => {
                attr <<= 2;
            }
            2 | 3 | 6 | 7 => {
                // NOP
                //attr >>= 2;
            }
            8 | 9 | 12 | 13 => {
                attr >>= 2;
            }
            10 | 11 | 14 | 15 => {
                attr >>= 4;
            }
            _ => {}
        }
        attr &= 0x0C;

        let pat = pat | attr;
        let col = self.mem[BG_PALETTE_BASE + pat as usize]; // [0..3F]
        let r = COLOR_TABLE[(col * 3 + 0) as usize];
        let g = COLOR_TABLE[(col * 3 + 1) as usize];
        let b = COLOR_TABLE[(col * 3 + 2) as usize];

        return (r, g, b);
    }

    fn generate_lut(&mut self) {
        // pattern table lut
        for hi in 0..=255 {
            for lo in 0..=255 {
                for x in 0..8 {
                    let pat_hi: u8 = (hi >> (7 - x)) & 0x01;
                    let pat_lo: u8 = (lo >> (7 - x)) & 0x01;
                    let pat: u8 = pat_hi << 1 | pat_lo;

                    let mut index: usize = hi as usize;
                    index *= 256;
                    index += lo as usize;
                    index *= 8;
                    index += x as usize;

                    self.pattern_lut[index] = pat;
                }
            }
        }

        // attribute table lut
        for v in 0..32 {
            for u in 0..32 {
                self.attr_addr_lut[v * 32 + u] = (v / 4) * 8 + u / 4;
            }
        }
    }
}
