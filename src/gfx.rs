use crate::GameVars;
use std::io;
use crossterm::{queue, cursor, style};


#[allow(nonstandard_style)]
#[non_exhaustive]
pub struct GFX_IDX;

#[allow(unused)]
#[allow(nonstandard_style)]
impl GFX_IDX {
    pub const CORRIDOR_FULLY_BLOCKED : usize = 0  ;
    pub const CORRIDOR_Uxxx          : usize = 1  ;
    pub const CORRIDOR_xRxx          : usize = 2  ;
    pub const CORRIDOR_URxx          : usize = 3  ;
    pub const CORRIDOR_xxDx          : usize = 4  ;
    pub const CORRIDOR_UxDx          : usize = 5  ;
    pub const CORRIDOR_xRDx          : usize = 6  ;
    pub const CORRIDOR_UxDR          : usize = 7  ;
    pub const CORRIDOR_xxxL          : usize = 8  ;
    pub const CORRIDOR_UxxL          : usize = 9  ;
    pub const CORRIDOR_xRxL          : usize = 10 ;
    pub const CORRIDOR_URxL          : usize = 11 ;
    pub const CORRIDOR_xxDL          : usize = 12 ;
    pub const CORRIDOR_UxDL          : usize = 13 ;
    pub const CORRIDOR_xRDL          : usize = 14 ;
    pub const CORRIDOR_URDL          : usize = 15 ;
    pub const CORRIDOR_UNKNOWN       : usize = 16 ; 
    pub const ENTITY_HERO            : usize = 17 ;
    pub const ENTITY_EXIT            : usize = 18 ;
    pub const DECORATION_SKULL       : usize = 19 ;
}

static GRAPHICS: [RawImage; 20] = [
    RawImage{desc:"X",debug:"",gfx:"#################################################################################",rows:9,cols:9},
    RawImage{desc:"dead end",debug:"v",gfx:"###...######...######...######...######...#######################################",rows:9,cols:9},
    RawImage{desc:"dead end",debug:"<",gfx:"##############################......###......###......###########################",rows:9,cols:9},
    RawImage{desc:"corridor",debug:"└",gfx:"###...######...######...######......###......###......###########################",rows:9,cols:9},
    RawImage{desc:"dead end",debug:"^",gfx:"##############################...######...######...######...######...######...###",rows:9,cols:9},
    RawImage{desc:"corridor",debug:"|",gfx:"###...######...######...######...######...######...######...######...######...###",rows:9,cols:9},
    RawImage{desc:"corridor",debug:"┌",gfx:"##############################......###......###......###...######...######...###",rows:9,cols:9},
    RawImage{desc:"intersection",debug:"├",gfx:"###...######...######...######......###......###......###...######...######...###",rows:9,cols:9},
    RawImage{desc:"dead end",debug:">",gfx:"###########################.....####.....####.....###############################",rows:9,cols:9},
    RawImage{desc:"corridor",debug:"┘",gfx:"###...######...######...###......###......###......##############################",rows:9,cols:9},
    RawImage{desc:"corridor",debug:"-",gfx:"###########################...........................###########################",rows:9,cols:9},
    RawImage{desc:"intersection",debug:"┴",gfx:"###...######...######...###...........................###########################",rows:9,cols:9},
    RawImage{desc:"corridor",debug:"┐",gfx:"###########################......###......###......######...######...######...###",rows:9,cols:9},
    RawImage{desc:"intersection",debug:"┤",gfx:"###...######...######...###......###......###......######...######...######...###",rows:9,cols:9},
    RawImage{desc:"intersection",debug:"┬",gfx:"###########################...........................###...######...######...###",rows:9,cols:9},
    RawImage{desc:"intersection",debug:"┼",gfx:"###...######...######...###...........................###...######...######...###",rows:9,cols:9},
    RawImage{desc: "", debug: "", gfx: "?????????????????????????????????????????????????????????????????????????????????", rows: 9, cols: 9 },
    RawImage{desc: "", debug: "", gfx: "@", rows: 1, cols: 1 },
    RawImage{desc: "", debug: "", gfx: "$", rows: 1, cols: 1 },
    RawImage{desc: "", debug: "", gfx: r"     ______        -'      '-    /            \ |              ||,   -    -   ,|| )(__/  \__)( ||/     /\     \|(_     ^^     _) \__|IIIIII|__/   |-\IIIIII/-|    \          /     `--------`                   ", rows: 13, cols: 16 },
];

struct RawImage {
    gfx: &'static str,
    #[allow(unused)]
    desc: &'static str,
    debug: &'static str,
    rows: usize,
    cols: usize,
}

#[derive(Clone)]
pub struct TerminalImage {
    gfx: String,
    rows: usize,
    cols: usize,
    pos_x: isize,
    pos_y: isize,
    end_x: isize,
    end_y: isize,
}

pub trait RenderableContent {
    fn render(&self, game: &mut GameVars, rows: usize, cols: usize) -> Vec<TerminalImage>;
}

pub struct TerminalWindow {
    imgs: Vec<TerminalImage>,
    pub content: Box<dyn RenderableContent>,
    pub rows: usize,
    pub cols: usize,
    pos_x: usize,
    pos_y: usize,
    vborder: Option<String>,
    hborder: Option<String>,
}

pub struct TerminalScreen {
    pub winds: Vec<TerminalWindow>,
    pub screen: io::Stdout
}

pub fn char_in_image(idx: usize, pos_x: usize, pos_y: usize) -> char {
    let raw = &GRAPHICS[idx];
    let idx = (raw.cols * pos_y) as usize + pos_x as usize;

    let detected_char = match raw.gfx.chars().nth(idx) {
        Some(c) => c,
        None => '#',
    };
    
    println!("detected {detected_char}");

    return detected_char;

}

impl TerminalImage {
    pub fn new(idx: usize, pos_x: isize, pos_y: isize) -> TerminalImage {
        let raw = &GRAPHICS[idx];
        
        return TerminalImage{
            gfx: raw.gfx.to_string(),
            rows: raw.rows,
            cols: raw.cols,
            pos_x: pos_x,
            pos_y: pos_y,
            end_x: pos_x as isize + raw.cols as isize,
            end_y: pos_y as isize + raw.rows as isize,
        }
    }

    pub fn with_text(text: String, pos_x: isize, pos_y: isize) -> TerminalImage {
        let len = text.len();
        TerminalImage {
            gfx: text,
            rows: 1,
            cols: len,
            pos_x,
            pos_y,
            end_x: pos_x + len as isize,
            end_y: pos_y + 1,
        }
    }

    pub fn with_debug_text(idx: usize, pos_x: isize, pos_y: isize) -> TerminalImage {
        let raw = &GRAPHICS[idx];

        TerminalImage {
            gfx: raw.debug.to_string(),
            rows: 1,
            cols: 1,
            pos_x,
            pos_y,
            end_x: pos_x + 1,
            end_y: pos_y + 1,
        }
    }
}

impl TerminalWindow {

    pub fn new(
        content: Box<dyn RenderableContent>,
        rows: usize,
        cols: usize,
        pos_x: usize,
        pos_y: usize,
        border: bool,
        bg: char
    ) -> TerminalWindow {
        let (hborder, vborder) = create_borders(cols, bg, border);

        return TerminalWindow{
            imgs: Vec::new(),
            content: content,
            rows: rows,
            cols: cols,
            pos_x: pos_x,
            pos_y: pos_y,
            hborder: hborder,
            vborder: vborder,
        }
    }
    
    pub fn push_images(&mut self, mut imgs: Vec<TerminalImage>) {
        self.imgs.append(&mut imgs);
    }

    pub fn clear(&mut self) {
        self.imgs.clear();
    }

}

impl TerminalScreen {
    pub fn new() -> TerminalScreen {
        return TerminalScreen{
            winds: Vec::new(),
            screen: io::stdout()
        }
    }

    pub fn add_new_window_to_layout<C: RenderableContent + 'static>(
        &mut self,
        content: C,
        rows: usize,
        cols: usize,
        pos_x: usize,
        pos_y: usize,
        border: bool,
        bg: char
    ) {
        self.winds.push(TerminalWindow::new(Box::new(content), rows, cols, pos_x, pos_y, border, bg));
    }
    
    pub fn render_window(&mut self, idx: usize) {
        let wind = &mut self.winds[idx];

        if wind.vborder.is_some() && wind.hborder.is_some() {
            for y in 0..wind.rows {
                queue!(self.screen, cursor::MoveTo( (wind.pos_x - 1) as u16, (wind.pos_y + y) as u16)).unwrap();
                queue!(self.screen, style::Print(&wind.vborder.as_ref().unwrap())).unwrap();
            }

            queue!(self.screen, cursor::MoveTo( (wind.pos_x - 1) as u16, (wind.pos_y - 1) as u16)).unwrap();
            queue!(self.screen, style::Print(&wind.hborder.as_ref().unwrap())).unwrap();
            queue!(self.screen, cursor::MoveTo( (wind.pos_x - 1) as u16, (wind.pos_y + wind.rows) as u16)).unwrap();
            queue!(self.screen, style::Print(&wind.hborder.as_ref().unwrap())).unwrap();
        }

        for img in &wind.imgs {

            let mut trim_left: isize = 0;
            let mut trim_up: isize = 0;
            let mut trim_down: usize = 0;
            let mut trim_right: usize = 0;

            if img.pos_x < 0 {
                trim_left = img.pos_x * -1;
            } 
            
            if img.pos_y < 0 {
                trim_up = img.pos_y * -1;
            }
            
            if img.end_x - wind.cols as isize >= 0 {
                trim_right = (img.end_x - wind.cols as isize) as usize;
                // assert!(trim_right <= img.cols, "end={}, cols={}", img.end_x, wind.cols);
            }

            if img.end_y - wind.rows as isize >= 0 {
                trim_down = (img.end_y - wind.rows as isize) as usize;
                // assert!(trim_down <= img.rows);
            }

            if trim_down >= img.rows {
                continue
            }

            for line in trim_up as usize..(img.rows - trim_down) {
                let left = line*img.cols + trim_left as usize;
                let right = line*img.cols + img.cols - trim_right;
                
                if left < right && right <= img.gfx.len() {
                    let img_line;
                    
                    // TODO: make debug and normal render using the same line here
                    // img_line = &img.gfx;
                    img_line = &img.gfx[left .. right];

                    queue!(self.screen, cursor::MoveTo(
                            (wind.pos_x as isize + img.pos_x + trim_left) as u16,
                            (wind.pos_y as isize + img.pos_y + line as isize) as u16,
                        )).unwrap();
                    queue!(self.screen, style::Print(img_line)).unwrap();
                }
            }
        }
    }
}

fn create_borders(cols: usize, bg: char, border: bool) -> (Option<String>, Option<String>) {
    if !border {
        return (None, None);
    }

    let mut h = String::new();
    let mut v = String::new();

    h.push_str("+");
    v.push_str("|");

    for _ in 0..cols {
        h.push_str("-");
        v.push(bg);
    }
    h.push_str("+");
    v.push_str("|");

    (Some(h), Some(v))
}
