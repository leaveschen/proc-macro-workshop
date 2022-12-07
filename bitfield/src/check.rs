
// Trait & Struct as tags.
pub trait DiscriminantInRange {}
pub struct True;
pub struct False;

impl DiscriminantInRange for True {}

pub trait Tag { type T; }

// For test-09
pub struct CheckRange<const V: usize, const B: bool>;

impl<const V: usize> Tag for CheckRange<V, true> {
    type T = True;
}

impl<const V: usize> Tag for CheckRange<V, false> {
    type T = False;
}

pub trait CheckRangeTrait<T: DiscriminantInRange> {}