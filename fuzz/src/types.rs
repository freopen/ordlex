use arbitrary::{Arbitrary, Unstructured};
use lexord::LexOrd;
use std::cmp::Ordering;

fn serialize_type<'a, T: LexOrd + Arbitrary<'a>>(
    data: &mut Unstructured<'a>,
    ser: &mut Vec<u8>,
) -> arbitrary::Result<T> {
    let result: T = data.arbitrary()?;
    let prev_pos = ser.len();
    result.to_write(ser).unwrap();
    let mut ser_slice = &ser[prev_pos..];
    let deser = T::from_read(&mut ser_slice).unwrap();
    assert_eq!(ser_slice.len(), 0);
    if let Some(ordering) = result.partial_cmp(&deser) {
        assert_eq!(ordering, Ordering::Equal);
    }
    Ok(result)
}

macro_rules! count {
    () => (0usize);
    ($head:tt $($tail:tt)*) => (1usize + count!($($tail)*));
}

pub trait FuzzType {
    fn test(
        &self,
        data: &mut Unstructured,
        ser_a: &mut Vec<u8>,
        ser_b: &mut Vec<u8>,
    ) -> arbitrary::Result<Option<Ordering>>;
}

macro_rules! gen_single_fns {
    ($($ty:ty)+) => {
        pub struct Single(u16);

        impl FuzzType for Single {
            #[allow(unused_assignments)]
            fn test(
                &self,
                data: &mut Unstructured,
                ser_a: &mut Vec<u8>,
                ser_b: &mut Vec<u8>
            ) -> arbitrary::Result<Option<Ordering>> {
                let mut type_id = self.0;
                $(
                    if type_id == 0 {
                        let a: $ty = serialize_type(data, ser_a)?;
                        let b: $ty = serialize_type(data, ser_b)?;
                        return Ok(a.partial_cmp(&b));
                    }
                    type_id -= 1;
                )+
                unreachable!()
            }
        }
        impl std::fmt::Debug for Single {
            #[allow(unused_assignments)]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut type_id = self.0;
                $(
                    if type_id == 0 {
                        return write!(f, "{}", stringify!($ty));
                    }
                    type_id -= 1;
                )+
                unreachable!()
            }
        }
        impl<'a> Arbitrary<'a> for Single {
            fn arbitrary(data: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
                let type_id = data.arbitrary()?;
                if type_id < count!($($ty)*) as u16 {
                    Ok(Single(type_id))
                } else {
                    Err(arbitrary::Error::IncorrectFormat)
                }
            }
        }
    }
}

gen_single_fns!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64 String);

#[derive(Debug, Arbitrary)]
pub enum TypeDef {
    Single(Single),
    Vec(Box<TypeDef>),
    Tuple(Vec<TypeDef>),
    Enum(Vec<(i64, TypeDef)>),
}

impl FuzzType for TypeDef {
    fn test(
        &self,
        data: &mut Unstructured,
        ser_a: &mut Vec<u8>,
        ser_b: &mut Vec<u8>,
    ) -> arbitrary::Result<Option<Ordering>> {
        match self {
            TypeDef::Single(single) => single.test(data, ser_a, ser_b),
            _ => Err(arbitrary::Error::IncorrectFormat),
        }
    }
}