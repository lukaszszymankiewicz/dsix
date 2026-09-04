use rand::Rng;

pub fn throw_k6_dice() -> usize {
    return rand::rng().random_range(0..6);
}
