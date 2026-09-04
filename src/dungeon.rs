use rand::prelude::IndexedRandom;

use crate::Dir;
use crate::gfx::GFX_IDX;
use crate::entity::{ENTITY_STAIRS_TO_LOWER_LEVEL, EntityType, Entity};
use crate::dice::throw_k6_dice;
use crate::gfx::char_in_image;

pub static BASE_ROOM_SIZE                           : usize = 9;

    static CHANCE_FOR_ROOM_TO_HAVE_ANY_EXIT_BLOCKED : usize = 3;
    static MAX_LEN_OF_PATH_TO_THE_EXIT              : usize = 6;
    static BASE_LEVEL_H                             : usize = 8;
    static BASE_LEVEL_W                             : usize = 8;

pub struct Room {
    visited: bool,
    exits: [bool; 4],
    img_idx: usize,
    entities: Vec<Entity>
}

pub struct Dungeon {
    pub level_number: usize,
    pub rows: usize,
    pub cols: usize,
    rooms: Vec<Room>
}

struct Path {
    steps: Vec<(usize, usize)>,
    max_len: usize,
    possible_next_steps: Vec<Dir>
}

impl Room {
    fn new() -> Room {
        let new_room: Room = Room{
            visited: false,
            exits: [true; 4],
            img_idx: GFX_IDX::CORRIDOR_UNKNOWN,
            entities: Vec::new()
        };

        return new_room;
    }

    fn update_the_associated_img_idx(&mut self) {
        let mut idx: usize = 0b0000_0000;

        for (index, bit) in self.exits.iter().enumerate().rev() {
            idx += (*bit as usize) << index;
        }

        self.img_idx = idx;
    }

    fn visit(&mut self) -> usize {
        if self.visited == true {
            return 0;
        }
        self.visited = true; 

        return 1;
    }

    // fn unvisit(&mut self) {
    //     self.visited = false; 
    // }

    fn block_exit(&mut self, dir: Dir) {
        self.exits[dir as usize] = false; 
    }

    fn unblock_exit(&mut self, dir: Dir) {
        self.exits[dir as usize] = true; 
    }

}

impl Dungeon {
    // DUNGEON GENERATE
    pub fn new(level_number: usize) -> Dungeon {
        let new_dungeon_cols = BASE_LEVEL_W + level_number;
        let new_dungeon_rows = BASE_LEVEL_H + level_number;
        let rooms_total = new_dungeon_cols * new_dungeon_rows;
        let mut new_rooms: Vec<Room> = Vec::new();
        
        // Dungeon coords start from upper-left corner
        for _ in 0..rooms_total {
            new_rooms.push(Room::new());
        }

        let new_dungeon = Dungeon {
            level_number: level_number,
            rows: new_dungeon_rows,
            cols: new_dungeon_cols,
            rooms: new_rooms
        };

        return new_dungeon;
    }

    pub fn init(&mut self) -> (usize, usize) {
        let init_row: usize = self.rows/2;
        let init_col: usize = self.cols/2;

        // block horizontal borders of whole level
        for col in 0..self.cols {
            self.block_all_exists_from_room(0, col);
            self.block_all_exists_from_room(self.rows-1, col);
        }

        // block vertical borders of whole level
        for row in 0..self.rows {
            self.block_all_exists_from_room(row, 0);
            self.block_all_exists_from_room(row, self.rows-1);
        }

        // generate random paths
        for row in 0..self.rows {
            for col in 0..self.cols {
                self.block_random_exit_of_room(row, col);
            }
        }
        
        for row in 0..self.rows {
            for col in 0..self.cols {
                self.get_room(row, col).update_the_associated_img_idx();
            }
        }

        self.unblock_all_exists_from_room(init_row, init_col);
        self.set_room_as_visited(init_row, init_col);
        self.spawn_an_entity_stairs_to_lower_level(init_row, init_col);

        // return the starting point of the hero, based on init room
        // TODO: this should be a part of placing an entity?
        return (
            init_row * BASE_ROOM_SIZE + BASE_ROOM_SIZE/2,
            init_col * BASE_ROOM_SIZE + BASE_ROOM_SIZE/2
        );
    }

    // DUNGEON ROOM
    fn get_room(&mut self, row: usize, col: usize) -> &mut Room {
        // rooms coords start from uppper-left corner of the Dungeon
        return self.rooms.get_mut(self.rows * row + col).unwrap()
    }

    fn get_room_by_entity_pos(&mut self, x: usize, y: usize) -> &mut Room {
        return self.get_room(y/BASE_ROOM_SIZE, x/BASE_ROOM_SIZE);
    }
    
    // DUNGEON GFX
    pub fn position_is_an_obstacle(&mut self, x: usize, y: usize) -> bool {
        let room_to_check: &Room = self.get_room_by_entity_pos(x, y);

        match char_in_image(room_to_check.img_idx, x % BASE_ROOM_SIZE, y % BASE_ROOM_SIZE){
            '#' => return true, 
            _ => return false
        }
    }

    pub fn get_the_image_idx_of_a_room(&mut self, row: usize, col: usize) -> usize {
        return self.get_room(row, col).img_idx;
    }
    
    // DUNGEON ROOM EXITS
    fn get_neighbor_room_coords(&self, row: usize, col: usize, dir: Dir) -> Option<(usize, usize)> {
        match dir {
            Dir::Left if col > 0 => Some((row, col - 1)),
            Dir::Right if col < self.cols - 1 => Some((row, col + 1)),
            Dir::Up if row > 0 => Some((row - 1, col)),
            Dir::Down if row < self.rows - 1 => Some((row + 1, col)),
            _ => None,
        }
    }

    fn block_single_exit_from_room(&mut self, row: usize, col: usize, dir: Dir) {
        self.get_room(row, col).block_exit(dir);

        if let Some((neighboour_row, neighbour_col)) = self.get_neighbor_room_coords(row, col, dir) {
            self.get_room(neighboour_row, neighbour_col).block_exit(dir.opposite());
        }
    }

    fn block_all_exists_from_room(&mut self, row: usize, col: usize) {
        for dir in [Dir::Left, Dir::Right, Dir::Up, Dir::Down] {
            self.block_single_exit_from_room(row, col, dir);
        }
    }

    fn unblock_all_exists_from_room(&mut self, row: usize, col: usize) {
        for dir in [Dir::Left, Dir::Right, Dir::Up, Dir::Down] {
            self.unblock_single_exit_from_room(row, col, dir);
        }
    }

    fn unblock_single_exit_from_room(&mut self, row: usize, col: usize, dir: Dir) {
        self.get_room(row, col).unblock_exit(dir);

        if let Some((neighboour_row, neighbour_col)) = self.get_neighbor_room_coords(row, col, dir) {
            self.get_room(neighboour_row, neighbour_col).unblock_exit(dir.opposite());
        }
    }

    fn block_random_exit_of_room(&mut self, row: usize, col: usize) {
        let exits = self.get_room(row, col).exits;

        // nothing more to block
        if exits == [false; 4] {
            return; 
        } 

        if throw_k6_dice() > CHANCE_FOR_ROOM_TO_HAVE_ANY_EXIT_BLOCKED {
            return; 
        }

        match throw_k6_dice() {
            0 => self.block_single_exit_from_room(row, col, Dir::Left),
            1 => self.block_single_exit_from_room(row, col, Dir::Right),
            2 => self.block_single_exit_from_room(row, col, Dir::Up),
            3 => self.block_single_exit_from_room(row, col, Dir::Down),
            4 => {
                self.block_single_exit_from_room(row, col, Dir::Up);
                self.block_single_exit_from_room(row, col, Dir::Down);
            },
            5 => {
                self.block_single_exit_from_room(row, col, Dir::Right);
                self.block_single_exit_from_room(row, col, Dir::Left);
            }
            _ => unreachable!()
        }
    }
    
    // DUNGEON VISIT
    fn set_room_as_visited(&mut self, row: usize, col: usize) -> usize {
        return self.get_room(row, col).visit();
    }
    
    pub fn room_is_visited(&mut self, row: usize, col: usize) -> bool {
        return self.get_room(row, col).visited;
    }
    
    pub fn visit_the_room_using_entity_pos(&mut self, x: usize, y: usize) -> usize {
        let room: &mut Room = self.get_room_by_entity_pos(x, y);

        if room.visited == false {
            room.visit();
            return 1;
        }

        return 0;
    }

    // DUNGEON DEBUG
    pub fn set_all_rooms_as_visited(&mut self) {
        for row in 0..self.rows {
            for col in 0..self.cols {
                let _ = self.get_room(row, col).visit();
            }
        }
    }
    
    // DUNGEON ENTITY
    pub fn get_n_entities_in_room(&mut self, row: usize, col: usize) -> usize {
        return self.get_room(row, col).entities.len();
    }

    pub fn get_entity_in_room_by_idx(&mut self, row: usize, col: usize, idx: usize) -> &Entity {
        return &self.get_room(row, col).entities[idx];
    }

    pub fn there_is_some_entities_in_the_room(&mut self, row: usize, col: usize) -> bool {
        return self.get_room(row, col).entities.len() > 0;
    }

    fn spawn_an_entity(&mut self, entity_type: EntityType, entity_pos_x: usize, entity_pos_y: usize) {
        let new_entity = Entity {
            entity_type: entity_type,
            entity_pos_x: entity_pos_x,
            entity_pos_y: entity_pos_y
        };
        
        // add it to the room
        let entity_room_row: usize = entity_pos_y / BASE_ROOM_SIZE;
        let entity_room_col: usize = entity_pos_x / BASE_ROOM_SIZE;

        self.get_room(entity_room_row, entity_room_col).entities.push(new_entity);
    }

    fn spawn_an_entity_stairs_to_lower_level(&mut self, initial_x: usize, initial_y: usize) {
        let mut new_path: Path = Path::new(initial_x, initial_y);

        // initally - all direction are possible, so any way can be chosen
        new_path.add_step_to_fixed_direction(Dir::Up);

        let mut last_step_x: usize;
        let mut last_step_y: usize;

        loop {

            let last_step = new_path.last_step();
            last_step_x = last_step.unwrap().0;
            last_step_y = last_step.unwrap().1;

            let possible_exits = self.get_room(last_step_x, last_step_y).exits;
            new_path.check_possible_next_steps(possible_exits);

            if new_path.possible_next_steps.len() == 0 || new_path.steps.len() >= new_path.max_len {
                break; 
            }

            let new_dir: Dir = new_path.choose_random_possible_step();
            new_path.add_step_to_fixed_direction(new_dir);
        }

        let exit_room = new_path.last_step();
        let exit_room_row = exit_room.unwrap().0;
        let exit_room_col = exit_room.unwrap().1;
        
        // place exit on the middle of a room
        let entity_pos_x = (exit_room_col * BASE_ROOM_SIZE) + (BASE_ROOM_SIZE / 2);
        let entity_pos_y = (exit_room_row * BASE_ROOM_SIZE) + (BASE_ROOM_SIZE / 2);

        self.spawn_an_entity(ENTITY_STAIRS_TO_LOWER_LEVEL, entity_pos_x, entity_pos_y);

    }
}

impl Path {
    fn new(initial_x: usize, initial_y: usize) -> Path {
        let new_path = Path {
            steps: vec![(initial_x, initial_y)],
            max_len: MAX_LEN_OF_PATH_TO_THE_EXIT,
            possible_next_steps: Vec::new()
        };
        return new_path;
    }

    fn last_step (&mut self) -> Option<(usize, usize)> {
        match self.steps.len() {
            0 => None,
            n => Some(self.steps[n-1])
        }
    }

    fn add_step_to_fixed_direction(&mut self, direction: Dir) {
        let last_step: (usize, usize) = self.last_step().unwrap();

        match direction {
            Dir::Left => {
                self.steps.push((last_step.0, last_step.1-1));
            }
            Dir::Right => {
                self.steps.push((last_step.0, last_step.1+1));
            }
            Dir::Up => {
                self.steps.push((last_step.0-1, last_step.1));
            }
            Dir::Down => {
                self.steps.push((last_step.0+1, last_step.1));
            }
        }
        
        self.possible_next_steps.clear();
    }

    fn check_possible_next_steps(&mut self, exits: [bool; 4]) {
        self.possible_next_steps.clear();

        let last_step = self.last_step();
        let last_step_x: usize = last_step.unwrap().0;
        let last_step_y: usize = last_step.unwrap().1;
        
        //urdl 

        if exits[3] == true & !self.steps.contains(&(last_step_x, last_step_y-1)) {
            self.possible_next_steps.push(Dir::Left);
        }

        if exits[1] == true & !self.steps.contains(&(last_step_x, last_step_y+1)) {
            self.possible_next_steps.push(Dir::Right);
        }

        if exits[0] == true & !self.steps.contains(&(last_step_x-1, last_step_y)) {
            self.possible_next_steps.push(Dir::Up);
        }
        
        if exits[2] == true & !self.steps.contains(&(last_step_x+1, last_step_y)) {
            self.possible_next_steps.push(Dir::Down);
        }

    }

    fn choose_random_possible_step(&mut self) -> Dir  {
        return *self.possible_next_steps.choose(&mut rand::rng()).unwrap();
    }

}


