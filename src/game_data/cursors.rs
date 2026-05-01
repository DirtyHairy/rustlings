use crate::game_data::Bitmap;

const WHITE: u8 = 3;
const GREEN: u8 = 2;
const BLUE: u8 = 1;

const CROSSHAIR: &str = "
......WG......
..............
......WG......
..............
......WG......
..............
G.G.G.BB.W.W.W
W.W.W.BB.G.G.G
..............
......GW......
..............
......GW......
..............
......GW......
";

const BOX: &str = "
GGWW..BB..WWGG
G............G
W............W
W............W
..............
..............
B............B
B............B
..............
..............
W............W
W............W
G............G
GGWW..BB..WWGG
";

pub const CURSOR_SIZE: u32 = 14;
pub const CURSOR_CENTER_X: f32 = 7.5;
pub const CURSOR_CENTER_Y: f32 = 7.5;
pub const CURSOR_NATIVE_SCREEN_WIDTH: u32 = 320;
pub const CURSOR_NATIVE_SCREEN_HEIGHT: u32 = 200;
pub const CURSOR_NATIVE_ASPECT: f32 = 1.2;

fn compile_char(c: char) -> Option<u8> {
    match c {
        'G' => Some(GREEN),
        'W' => Some(WHITE),
        'B' => Some(BLUE),
        '.' => Some(0),
        _ => None,
    }
}

fn compile(input: &str) -> Bitmap {
    let cursor_size = CURSOR_SIZE as usize;
    let mut data: Vec<u8> = vec![0; cursor_size * cursor_size];
    let mut transparency: Vec<bool> = vec![true; cursor_size * cursor_size];

    let mut i = 0;
    for c in input.chars() {
        let compiled = compile_char(c);
        if let Some(color) = compiled {
            if color > 0 {
                data[i] = color;
                transparency[i] = false;
            }

            i += 1;
        }
    }

    Bitmap {
        width: cursor_size,
        height: cursor_size,
        data,
        transparency,
    }
}

#[derive(Clone)]
pub struct Cursors {
    pub crosshair: Bitmap,
    pub boxx: Bitmap,
}

impl Cursors {
    pub fn new() -> Self {
        let crosshair = compile(CROSSHAIR);
        let boxx = compile(BOX);

        Self { crosshair, boxx }
    }
}
