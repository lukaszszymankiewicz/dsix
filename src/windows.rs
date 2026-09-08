use crate::dungeon::BASE_ROOM_SIZE;
use crate::game::GameVars;
use crate::gfx::{TerminalImage, RenderableContent, GFX_IDX};


pub struct WindowMainMapContents;
impl RenderableContent for WindowMainMapContents {
    fn render(&self, vars: &mut GameVars, rows: usize, cols: usize) -> Vec<TerminalImage> {
        let win_h = rows;
        let win_w = cols;
        let mut imgs = Vec::new();

        let camera_st_x = if vars.hero_pos_x >= win_w / 2 { vars.hero_pos_x - win_w / 2 } else { 0 };
        let camera_end_x = vars.hero_pos_x + win_w / 2;
        let camera_st_y = if vars.hero_pos_y >= win_h / 2 { vars.hero_pos_y - win_h / 2 } else { 0 };
        let camera_end_y = vars.hero_pos_y + win_h / 2;

        let st_cell_left = camera_st_x / BASE_ROOM_SIZE;
        let end_cell_right = camera_end_x / BASE_ROOM_SIZE;
        let st_cell_up = camera_st_y / BASE_ROOM_SIZE;
        let end_cell_down = camera_end_y / BASE_ROOM_SIZE;
        
        // get room images and their position on the window
        // x_pad and y_pad are calculated to know if window slices the image (from left and up).
        // sliced image is not fully rendered (obviously)
        for row in st_cell_up..end_cell_down+1 {
            for col in st_cell_left..end_cell_right+1 {
                
                let x_pad: isize = (vars.hero_pos_x as isize) - (win_w / 2) as isize;
                let y_pad: isize = (vars.hero_pos_y as isize) - (win_h / 2) as isize;

                let x = ((col * BASE_ROOM_SIZE) as isize) - x_pad;
                let y = ((row * BASE_ROOM_SIZE) as isize) - y_pad;
                
                let idx: usize = vars.dungeon.get_the_image_idx_of_a_room(row, col);

                // FOG OF WAR
                if !vars.dungeon.room_is_visited(row, col) {
                    imgs.push(TerminalImage::new(GFX_IDX::CORRIDOR_UNKNOWN, x, y));
                } else {
                    imgs.push(TerminalImage::new(idx, x, y));
                }

                // if there is no entity to render, just follow along
                if vars.dungeon.room_is_visited(row, col) {
                    continue
                }

                for idx in 0..vars.dungeon.get_n_entities_in_room(row, col) {

                    let ent = vars.dungeon.get_entity_in_room_by_idx(row, col, idx);

                    let entity_render_x = ent.entity_pos_x as isize - x_pad;
                    let entity_render_y = ent.entity_pos_y as isize - y_pad;
                    let img = ent.entity_type.img_idx;

                    imgs.push(TerminalImage::new(img, entity_render_x, entity_render_y));
                };
            }
        }

        // Hero
        let pos_x = win_w as isize / 2;
        let pos_y = win_h as isize / 2;

        imgs.push(TerminalImage::new(GFX_IDX::ENTITY_HERO, pos_x, pos_y));
        
        return imgs;
    }
}

pub struct DebugWindowMainMapContents;
impl RenderableContent for DebugWindowMainMapContents {
    fn render(&self, vars: &mut GameVars, _rows: usize, _cols: usize) -> Vec<TerminalImage> {
        
        let mut imgs = Vec::new();

        for row in 0..vars.dungeon.rows {
            for col in 0..vars.dungeon.cols {

                let idx: usize = vars.dungeon.get_the_image_idx_of_a_room(row, col);

                imgs.push(TerminalImage::with_debug_text(idx, col.try_into().unwrap(), row.try_into().unwrap()));
            }
        }

        // Hero
        imgs.push(TerminalImage::new(
            GFX_IDX::ENTITY_HERO,
            (vars.dungeon.cols/2).try_into().unwrap(),
            (vars.dungeon.rows/2).try_into().unwrap())
        );

        imgs 
    }
}

pub struct GreetingWindowContent;
impl RenderableContent for GreetingWindowContent {
    fn render(&self, game: &mut GameVars, _rows: usize, _cols: usize) -> Vec<TerminalImage> {
        let mut imgs = Vec::new();

        // Values
        imgs.push(TerminalImage::with_text("LEVEL 1".to_string(), 12, 6));
        imgs.push(TerminalImage::with_text("(press any key)".to_string(), 9, 8));

        imgs 
    }
}
pub struct WindowHeroStatsContent;
impl RenderableContent for WindowHeroStatsContent {
    fn render(&self, game: &mut GameVars, _rows: usize, _cols: usize) -> Vec<TerminalImage> {
        let mut imgs = Vec::new();

        // Labels
        imgs.push(TerminalImage::with_text("LEVEL".to_string(), 1, 1));
        imgs.push(TerminalImage::with_text("ATTACK".to_string(), 1, 3));
        imgs.push(TerminalImage::with_text("ARMOR".to_string(), 1, 4));
        imgs.push(TerminalImage::with_text("SPEED".to_string(), 1, 5));
        imgs.push(TerminalImage::with_text("EXP".to_string(), 1, 6));
        imgs.push(TerminalImage::with_text("ROW".to_string(), 1, 7));
        imgs.push(TerminalImage::with_text("COL".to_string(), 1, 8));

        // Values
        imgs.push(TerminalImage::with_text(game.dungeon.level_number.to_string(), 10, 1));
        imgs.push(TerminalImage::with_text(game.attack.to_string(), 10, 3));
        imgs.push(TerminalImage::with_text(game.armor.to_string(), 10, 4));
        imgs.push(TerminalImage::with_text(game.speed.to_string(), 10, 5));
        imgs.push(TerminalImage::with_text(game.exp.to_string(), 10, 6));
        imgs.push(TerminalImage::with_text(game.hero_pos_x.to_string(), 10, 7));
        imgs.push(TerminalImage::with_text(game.hero_pos_y.to_string(), 10, 8));

        imgs 
    }
}

pub struct WindowSkullImageContent;
impl RenderableContent for WindowSkullImageContent {
    fn render(&self, _game: &mut GameVars, _rows: usize, _cols: usize) -> Vec<TerminalImage> {
        let mut imgs = Vec::new();
        imgs.push(TerminalImage::new(GFX_IDX::DECORATION_SKULL, 2, 0));
        imgs 
    }
}

pub struct WindowTopBannerContent;
impl RenderableContent for WindowTopBannerContent {
    fn render(&self, _game: &mut GameVars, _rows: usize, _cols: usize) -> Vec<TerminalImage> {
        let mut imgs = Vec::new();
        imgs.push(TerminalImage::with_text("EXPLORATION".to_string(), 32, 0));
        imgs 
    }
}

pub struct WindowLogsContent;
impl RenderableContent for WindowLogsContent {
    fn render(&self, vars: &mut GameVars, _rows: usize, _cols: usize) -> Vec<TerminalImage> {
        let log_size: isize = 2;
        let mut i: isize = 0;
        let mut imgs = Vec::new();
        
        // inital logs
        if vars.logs.len() == 0 {

            for row in 0..vars.dungeon.rows {
                for col in 0..vars.dungeon.cols {
                        
                    let mut exit_coord_string: String = String::new();
                    exit_coord_string.push_str(&col.to_string());
                    exit_coord_string.push(',');
                    exit_coord_string.push_str(&row.to_string());
                    
                    // this is just a debug info, it should not be used anywhere else
                    if vars.dungeon.there_is_some_entities_in_the_room(row, col) {
                        imgs.push(TerminalImage::with_text(exit_coord_string, 0, 0));
                    }
                }
            }
        }

        for s in vars.logs.iter().rev() { 

            if i > log_size { break; }

            imgs.push(TerminalImage::with_text(s.to_string(), 0, log_size-i));
            i+=1;
        }

        return imgs;
    }
}
