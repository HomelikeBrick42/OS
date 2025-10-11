#![no_std]

use core::ffi::CStr;

const SPACE_MONO_FNT: &[u8] = include_bytes!("./font_data/space_mono.fnt");
const SPACE_MONO_TGA_0: &[u8] = include_bytes!("./font_data/space_mono_0.tga");

pub struct Info<'a> {
    pub font_size: u16,
    pub smooth: bool,
    pub unicode: bool,
    pub italic: bool,
    pub bold: bool,
    pub fixed_height: bool,
    pub char_set: u8,
    pub stretch_h: u16,
    pub aa: bool,
    pub padding_up: u8,
    pub padding_right: u8,
    pub padding_down: u8,
    pub padding_left: u8,
    pub spacing_horiz: u8,
    pub spacing_vert: u8,
    pub outline: u8,
    pub font_name: &'a CStr,
}

pub struct Common {
    pub line_height: u16,
    pub base: u16,
    pub scale_w: u16,
    pub scale_h: u16,
    pub pages: u16,
    pub packed: bool,
    pub alpha_channel: u8,
    pub red_channel: u8,
    pub green_channel: u8,
    pub blue_channel: u8,
}

pub struct Char {
    pub id: u32,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub xoffset: u16,
    pub yoffset: u16,
    pub xadvance: u16,
    pub page: u8,
    pub chnl: u8,
}

pub struct Page<'a> {
    pub width: u16,
    pub height: u16,
    pub brightnesses: &'a [u8],
}

pub struct Font<'a> {
    pub info: Info<'a>,
    pub common: Common,
    pub chars: &'a [Char],
    pub pages: &'a [Page<'a>],
}

pub const SPACE_MONO: Font<'static> = Font {
    info: parse_info(SPACE_MONO_FNT),
    common: parse_common(SPACE_MONO_FNT),
    chars: &parse_chars::<{ chars_count(SPACE_MONO_FNT) }>(SPACE_MONO_FNT),
    pages: &[parse_page(SPACE_MONO_TGA_0)],
};

const fn parse_info(bytes: &[u8]) -> Info<'_> {
    let block = find_block(bytes, 1);
    Info {
        font_size: u16::from_ne_bytes([block[0], block[1]]),
        smooth: block[2] & (1 << 0) != 0,
        unicode: block[2] & (1 << 1) != 0,
        italic: block[2] & (1 << 2) != 0,
        bold: block[2] & (1 << 3) != 0,
        fixed_height: block[2] & (1 << 4) != 0,
        char_set: block[3],
        stretch_h: u16::from_ne_bytes([block[4], block[5]]),
        aa: block[6] != 0,
        padding_up: block[7],
        padding_right: block[8],
        padding_down: block[9],
        padding_left: block[10],
        spacing_horiz: block[11],
        spacing_vert: block[12],
        outline: block[13],
        font_name: match CStr::from_bytes_until_nul(block.split_at(14).1) {
            Ok(name) => name,
            Err(_) => panic!(),
        },
    }
}

const fn parse_common(bytes: &[u8]) -> Common {
    let block = find_block(bytes, 2);
    Common {
        line_height: u16::from_ne_bytes([block[0], block[1]]),
        base: u16::from_ne_bytes([block[2], block[3]]),
        scale_w: u16::from_ne_bytes([block[4], block[5]]),
        scale_h: u16::from_ne_bytes([block[6], block[7]]),
        pages: u16::from_ne_bytes([block[8], block[9]]),
        packed: block[10] & (1 << 7) != 0,
        alpha_channel: block[11],
        red_channel: block[12],
        green_channel: block[13],
        blue_channel: block[14],
    }
}

const fn chars_count(bytes: &[u8]) -> usize {
    let block = find_block(bytes, 4);
    block.len() / 20
}

const fn parse_chars<const N: usize>(bytes: &[u8]) -> [Char; N] {
    let mut chars = [const {
        Char {
            id: 0,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            xoffset: 0,
            yoffset: 0,
            xadvance: 0,
            page: 0,
            chnl: 0,
        }
    }; _];

    let block = find_block(bytes, 4);

    {
        let mut i = 0;
        while i < N {
            let index = i * 20;
            chars[i] = Char {
                id: u32::from_ne_bytes([
                    block[index],
                    block[index + 1],
                    block[index + 2],
                    block[index + 3],
                ]),
                x: u16::from_ne_bytes([block[index + 4], block[index + 5]]),
                y: u16::from_ne_bytes([block[index + 6], block[index + 7]]),
                width: u16::from_ne_bytes([block[index + 8], block[index + 9]]),
                height: u16::from_ne_bytes([block[index + 10], block[index + 11]]),
                xoffset: u16::from_ne_bytes([block[index + 12], block[index + 13]]),
                yoffset: u16::from_ne_bytes([block[index + 14], block[index + 15]]),
                xadvance: u16::from_ne_bytes([block[index + 16], block[index + 17]]),
                page: block[18],
                chnl: block[19],
            };
            i += 1;
        }
    }

    // sort the chars so a binary search can be done later
    {
        let mut i = 1;
        while i < chars.len() {
            let mut j = i;
            while j > 0 && chars[j - 1].id > chars[j].id {
                chars.swap(j - 1, j);
                j -= 1;
            }
            i += 1;
        }
    }

    chars
}

const fn find_block(bytes: &[u8], id: u8) -> &[u8] {
    let mut index = 0;

    assert!(bytes[index] == b'B');
    index += 1;
    assert!(bytes[index] == b'M');
    index += 1;
    assert!(bytes[index] == b'F');
    index += 1;
    assert!(bytes[index] == 3);
    index += 1;

    while index < bytes.len() {
        let found_id = bytes[index];
        index += 1;
        let length = u32::from_ne_bytes([
            bytes[index],
            bytes[index + 1],
            bytes[index + 2],
            bytes[index + 3],
        ]) as usize;
        index += 4;

        let start = index;
        index += length;
        if found_id == id {
            let start = bytes.split_at(start).1;
            return match start.split_at_checked(index) {
                Some((s, _)) => s,
                None => start,
            };
        }
    }

    panic!("there was no block with that id")
}

const fn parse_page(bytes: &[u8]) -> Page<'_> {
    let mut index = 0;

    // id length
    assert!(bytes[index] == 0);
    index += 1;

    // color map
    assert!(bytes[index] == 0);
    index += 1;

    // image type
    assert!(bytes[index] == 3);
    index += 1;

    // color map first index
    index += 2;

    // color map length
    assert!(bytes[index] == 0);
    assert!(bytes[index + 1] == 0);
    index += 2;

    // origin
    index += 4;

    let width = u16::from_le_bytes([bytes[index], bytes[index + 1]]);
    index += 2;
    let height = u16::from_le_bytes([bytes[index], bytes[index + 1]]);
    index += 2;

    // bits per pixel
    index += 1;

    // image descriptor
    index += 1;

    let brightnesses = bytes
        .split_at(index)
        .1
        .split_at(width as usize * height as usize)
        .0;
    Page {
        width,
        height,
        brightnesses,
    }
}
