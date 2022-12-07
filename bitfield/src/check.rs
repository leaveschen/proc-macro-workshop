
// Trait & Struct as tags.
pub trait TotalSizeIsMultipleOfEightBits {}
pub struct ZeroMod8;
pub struct OneMod8;
pub struct TwoMod8;
pub struct ThreeMod8;
pub struct FourMod8;
pub struct FiveMod8;
pub struct SixMod8;
pub struct SevenMod8;
impl TotalSizeIsMultipleOfEightBits for ZeroMod8 {}


pub trait DiscriminantInRange {}
pub struct True;
pub struct False;

impl DiscriminantInRange for True {}

pub trait Tag { type T; }

// For test 09.
pub struct CheckRange<const V: usize, const B: bool>;
impl<const V: usize> Tag for CheckRange<V, true> { type T = True; }
impl<const V: usize> Tag for CheckRange<V, false> { type T = False; }

pub trait CheckRangeTrait<T: DiscriminantInRange> {}

// For test 04.
pub struct CheckMod<const M: usize>;
impl Tag for CheckMod<0> { type T = ZeroMod8; }
impl Tag for CheckMod<1> { type T = OneMod8; }
impl Tag for CheckMod<2> { type T = TwoMod8; }
impl Tag for CheckMod<3> { type T = ThreeMod8; }
impl Tag for CheckMod<4> { type T = FourMod8; }
impl Tag for CheckMod<5> { type T = FiveMod8; }
impl Tag for CheckMod<6> { type T = SixMod8; }
impl Tag for CheckMod<7> { type T = SevenMod8; }

pub trait CheckModTrait<T: TotalSizeIsMultipleOfEightBits> {}