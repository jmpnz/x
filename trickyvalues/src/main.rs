mod boxed_value {

    struct BoxedValue(f64);

    impl BoxedValue {
        pub fn from_f64(value: f64) -> Self {
            Self {
                0: f64::from_ne_bytes(value.to_ne_bytes()),
            }
        }
        pub fn from_f32(value: f32) -> Self {
            Self {
                0: f64::from_ne_bytes((value as f64).to_ne_bytes()),
            }
        }
        pub fn from_i64(value: i64) -> Self {
            Self {
                0: f64::from_ne_bytes(value.to_ne_bytes()),
            }
        }
        pub fn from_i32(value: i32) -> Self {
            Self {
                //0: f64::from_ne_bytes((value).to_ne_bytes()),
                0: f64::from_bits(value as u32 as u64),
            }
        }

        pub fn to_f64(&self) -> f64 {
            f64::from_ne_bytes(self.0.to_ne_bytes())
        }
        pub fn to_f32(&self) -> f32 {
            // lossy caller is responsible for correctness
            f32::from_ne_bytes((self.0 as f32).to_ne_bytes())
        }
        pub fn to_i64(&self) -> i64 {
            i64::from_ne_bytes(self.0.to_ne_bytes())
        }
        pub fn to_i32(&self) -> i32 {
            self.0.to_bits() as u32 as i32
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::boxed_value::BoxedValue;

        #[test]
        fn can_handle_arbitrary_data_types() {
            let a = -12348549845454i64;
            let b = -3.1459349608734023614f64;
            let c = 0xcafebabei64;
            let d = 0xdeadbeefi64;
            let e = -98645345i32;
            let f = -365.364f32;

            assert_eq!(BoxedValue::from_i64(a).to_i64(), a);
            assert_eq!(BoxedValue::from_f64(b).to_f64(), b);
            assert_eq!(BoxedValue::from_i64(c).to_i64(), c);
            assert_eq!(BoxedValue::from_i64(d).to_i64(), d);
            assert_eq!(BoxedValue::from_i32(e).to_i32(), e);
            assert_eq!(BoxedValue::from_f32(f).to_f32(), f);
        }
    }
}

fn main() {}
