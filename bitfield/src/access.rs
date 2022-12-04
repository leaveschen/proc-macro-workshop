// Helper trait to implement get/set method.
// Implement this trait for type u8, u16, u32, etc.
pub trait Access<const BITS: usize, const OFFSET: usize, const TOTAL: usize>
where Self: Sized + std::ops::Not {
    // Bits of value with type `Self`.
    const VSIZE: usize = std::mem::size_of::<Self>() * 8;
    const BYTE_BEGIN: usize = if OFFSET + Self::VSIZE <= TOTAL {
        OFFSET / 8
    } else {
        OFFSET / Self::VSIZE
    };
    const BYTE_END: usize = (OFFSET + BITS - 1) / 8;
    const CROSS: bool = Self::BYTE_END - Self::BYTE_BEGIN + 1 > Self::VSIZE / 8;
    const SET: fn(data: &mut [u8], x: Self) = if Self::CROSS { Self::set_cross } else { Self::set_no_cross };
    const GET: fn(data: &[u8]) -> Self = if Self::CROSS { Self::get_cross } else { Self::get_no_cross };
    const OFFSET_BIT: usize = OFFSET - Self::BYTE_BEGIN * 8;
    const HIGHER_BIT: usize = BITS - (Self::VSIZE - Self::OFFSET_BIT); // for cross get/set
    const MASK: Self;
    const HIGHER_MASK: u8 = u8::MAX >> (8 - Self::HIGHER_BIT); // for cross get/set

    fn set_no_cross(data: &mut [u8], x: Self);
    fn set_cross(data: &mut [u8], x: Self);
    fn get_no_cross(data: &[u8]) -> Self;
    fn get_cross(data: &[u8]) -> Self;
}

// Impl Access for u8
impl<const BITS: usize, const OFFSET: usize, const TOTAL: usize> Access<BITS, OFFSET, TOTAL> for u8 {
    const MASK: Self = (u8::MAX >> (8 - BITS)) << <Self as Access<BITS, OFFSET, TOTAL>>::OFFSET_BIT;

    fn set_no_cross(data: &mut [u8], x: Self) {
        let slice = &mut data[<Self as Access<BITS, OFFSET, TOTAL>>::BYTE_BEGIN..
            <Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END+1];
        let old = u8::from_ne_bytes(slice.try_into().unwrap());
        let new = (old & !<Self as Access<BITS, OFFSET, TOTAL>>::MASK) |
            (x << <Self as Access<BITS, OFFSET, TOTAL>>::OFFSET_BIT);
        slice.copy_from_slice(&new.to_ne_bytes());
    }

    fn set_cross(data: &mut [u8], x: Self) {
        // lower bits
        let slice = &mut data[<Self as Access<BITS, OFFSET, TOTAL>>::BYTE_BEGIN..
            <Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END];
        let old = u8::from_ne_bytes(slice.try_into().unwrap());
        let new = (old & !<Self as Access<BITS, OFFSET, TOTAL>>::MASK) |
            (x << <Self as Access<BITS, OFFSET, TOTAL>>::OFFSET_BIT);
        slice.copy_from_slice(&new.to_ne_bytes());
        // higher bits
        let slice = &mut data[<Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END..
            <Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END+1];
        let old = u8::from_ne_bytes(slice.try_into().unwrap());
        let higher_x = (x >> (<Self as Access<BITS, OFFSET, TOTAL>>::VSIZE - <Self as Access<BITS, OFFSET, TOTAL>>::OFFSET_BIT)) as u8;
        let new = (old & !<Self as Access<BITS, OFFSET, TOTAL>>::HIGHER_MASK) | higher_x;
        slice.copy_from_slice(&new.to_ne_bytes());
    }

    fn get_no_cross(data: &[u8]) -> Self {
        let slice = &data[<Self as Access<BITS, OFFSET, TOTAL>>::BYTE_BEGIN..
            <Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END+1];
        let value = u8::from_ne_bytes(slice.try_into().expect("slice to array error"));
        (value & <Self as Access<BITS, OFFSET, TOTAL>>::MASK) >>
            <Self as Access<BITS, OFFSET, TOTAL>>::OFFSET_BIT
    }

    fn get_cross(data: &[u8]) -> Self {
        // lower bits
        let slice = &data[<Self as Access<BITS, OFFSET, TOTAL>>::BYTE_BEGIN..
            <Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END];
        let mut value = u8::from_ne_bytes(slice.try_into().unwrap());
        value = (value & <Self as Access<BITS, OFFSET, TOTAL>>::MASK) >> <Self as Access<BITS, OFFSET, TOTAL>>::OFFSET_BIT;
        // higher bits
        let slice = &data[<Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END..
            <Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END+1];
        let higher = u8::from_ne_bytes(slice.try_into().unwrap());
        let higher = (higher & <Self as Access<BITS, OFFSET, TOTAL>>::HIGHER_MASK) as Self;
        value | (higher << 8 - <Self as Access<BITS, OFFSET, TOTAL>>::OFFSET_BIT)
    }
}


// Impl Access for u16
impl<const BITS: usize, const OFFSET: usize, const TOTAL: usize> Access<BITS, OFFSET, TOTAL> for u16 {
    const MASK: Self = (u16::MAX >> (16 - BITS)) << <Self as Access<BITS, OFFSET, TOTAL>>::OFFSET_BIT;

    fn set_no_cross(data: &mut [u8], x: Self) {
        let slice = &mut data[<Self as Access<BITS, OFFSET, TOTAL>>::BYTE_BEGIN..
            <Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END+1];
        let old = u16::from_ne_bytes(slice.try_into().unwrap());
        let new = (old & !<Self as Access<BITS, OFFSET, TOTAL>>::MASK) |
            (x << <Self as Access<BITS, OFFSET, TOTAL>>::OFFSET_BIT);
        slice.copy_from_slice(&new.to_ne_bytes());
    }

    fn set_cross(data: &mut [u8], x: Self) {
        // lower bits
        let slice = &mut data[<Self as Access<BITS, OFFSET, TOTAL>>::BYTE_BEGIN..
            <Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END];
        let old = u16::from_ne_bytes(slice.try_into().unwrap());
        let new = (old & !<Self as Access<BITS, OFFSET, TOTAL>>::MASK) |
            (x << <Self as Access<BITS, OFFSET, TOTAL>>::OFFSET_BIT);
        slice.copy_from_slice(&new.to_ne_bytes());
        // higher bits
        let slice = &mut data[<Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END..
            <Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END+1];
        let old = u8::from_ne_bytes(slice.try_into().unwrap());
        let higher_x = (x >> (<Self as Access<BITS, OFFSET, TOTAL>>::VSIZE - <Self as Access<BITS, OFFSET, TOTAL>>::OFFSET_BIT)) as u8;
        let new = (old & !<Self as Access<BITS, OFFSET, TOTAL>>::HIGHER_MASK) | higher_x;
        slice.copy_from_slice(&new.to_ne_bytes());
    }

    fn get_no_cross(data: &[u8]) -> Self {
        let slice = &data[<Self as Access<BITS, OFFSET, TOTAL>>::BYTE_BEGIN..
            <Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END+1];
        let value = u16::from_ne_bytes(slice.try_into().expect("slice to array error"));
        (value & <Self as Access<BITS, OFFSET, TOTAL>>::MASK) >>
            <Self as Access<BITS, OFFSET, TOTAL>>::OFFSET_BIT
    }

    fn get_cross(data: &[u8]) -> Self {
        // lower bits
        let slice = &data[<Self as Access<BITS, OFFSET, TOTAL>>::BYTE_BEGIN..
            <Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END];
        let mut value = u16::from_ne_bytes(slice.try_into().unwrap());
        value = (value & <Self as Access<BITS, OFFSET, TOTAL>>::MASK) >> <Self as Access<BITS, OFFSET, TOTAL>>::OFFSET_BIT;
        // higher bits
        let slice = &data[<Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END..
            <Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END+1];
        let higher = u8::from_ne_bytes(slice.try_into().unwrap());
        let higher = (higher & <Self as Access<BITS, OFFSET, TOTAL>>::HIGHER_MASK) as Self;
        value | (higher << 16 - <Self as Access<BITS, OFFSET, TOTAL>>::OFFSET_BIT)
    }
}


// Impl Access for u32
impl<const BITS: usize, const OFFSET: usize, const TOTAL: usize> Access<BITS, OFFSET, TOTAL> for u32 {
    const MASK: Self = (u32::MAX >> (32 - BITS)) << <Self as Access<BITS, OFFSET, TOTAL>>::OFFSET_BIT;

    fn set_no_cross(data: &mut [u8], x: Self) {
        let slice = &mut data[<Self as Access<BITS, OFFSET, TOTAL>>::BYTE_BEGIN..
            <Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END+1];
        let old = u32::from_ne_bytes(slice.try_into().unwrap());
        let new = (old & !<Self as Access<BITS, OFFSET, TOTAL>>::MASK) |
            (x << <Self as Access<BITS, OFFSET, TOTAL>>::OFFSET_BIT);
        slice.copy_from_slice(&new.to_ne_bytes());
    }

    fn set_cross(data: &mut [u8], x: Self) {
        // lower bits
        let slice = &mut data[<Self as Access<BITS, OFFSET, TOTAL>>::BYTE_BEGIN..
            <Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END];
        let old = u32::from_ne_bytes(slice.try_into().unwrap());
        let new = (old & !<Self as Access<BITS, OFFSET, TOTAL>>::MASK) |
            (x << <Self as Access<BITS, OFFSET, TOTAL>>::OFFSET_BIT);
        slice.copy_from_slice(&new.to_ne_bytes());
        // higher bits
        let slice = &mut data[<Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END..
            <Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END+1];
        let old = u8::from_ne_bytes(slice.try_into().unwrap());
        let higher_x = (x >> (<Self as Access<BITS, OFFSET, TOTAL>>::VSIZE - <Self as Access<BITS, OFFSET, TOTAL>>::OFFSET_BIT)) as u8;
        let new = (old & !<Self as Access<BITS, OFFSET, TOTAL>>::HIGHER_MASK) | higher_x;
        slice.copy_from_slice(&new.to_ne_bytes());
    }

    fn get_no_cross(data: &[u8]) -> Self {
        let slice = &data[<Self as Access<BITS, OFFSET, TOTAL>>::BYTE_BEGIN..
            <Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END+1];
        let value = u32::from_ne_bytes(slice.try_into().expect("slice to array error"));
        (value & <Self as Access<BITS, OFFSET, TOTAL>>::MASK) >>
            <Self as Access<BITS, OFFSET, TOTAL>>::OFFSET_BIT
    }

    fn get_cross(data: &[u8]) -> Self {
        // lower bits
        let slice = &data[<Self as Access<BITS, OFFSET, TOTAL>>::BYTE_BEGIN..
            <Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END];
        let mut value = u32::from_ne_bytes(slice.try_into().unwrap());
        value = (value & <Self as Access<BITS, OFFSET, TOTAL>>::MASK) >> <Self as Access<BITS, OFFSET, TOTAL>>::OFFSET_BIT;
        // higher bits
        let slice = &data[<Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END..
            <Self as Access<BITS, OFFSET, TOTAL>>::BYTE_END+1];
        let higher = u8::from_ne_bytes(slice.try_into().unwrap());
        let higher = (higher & <Self as Access<BITS, OFFSET, TOTAL>>::HIGHER_MASK) as Self;
        value | (higher << 32 - <Self as Access<BITS, OFFSET, TOTAL>>::OFFSET_BIT)
    }
}