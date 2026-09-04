use crate::gfx::GFX_IDX;

pub struct Entity {
    pub entity_type: EntityType,
    pub entity_pos_x: usize,
    pub entity_pos_y: usize
}

pub struct EntityType{
    pub img_idx: usize,
}

pub const ENTITY_STAIRS_TO_LOWER_LEVEL: EntityType = EntityType{img_idx: GFX_IDX::ENTITY_EXIT};
