use std::str::FromStr;
use num_traits::cast::FromPrimitive;

pub struct Utility {}

impl Utility {
    pub fn convert_to_microseconds(time_string: &String) -> u64 {
        let mut time = String::new();
        let mut units = String::new();
        for char in time_string.chars() {
            if char.is_alphabetic() {
                units.push(char);
            } else {
                time.push(char);
            }
        }
        let time = time.trim();
        let units = units.trim();
        let time_float = f64::from_str(time)
            .expect("[Utility::convert_to_microseconds]: Failed to parse time_float from time String");
        match units {
            "us" => u64::from_f64(time_float)
                .expect("[Utility::convert_to_microseconds]: Failed to convert time_float to u64"),
            "ms" => u64::from_f64(time_float * 1000.0)
                .expect("[Utility::convert_to_microseconds]: Failed to convert time_float to u64"),
            "s" => u64::from_f64(time_float * 1000000.0)
                .expect("[Utility::convert_to_microseconds]: Failed to convert time_float to u64"),
            _ => panic!("[Utility::convert_to_microseconds]: Invalid unit {units}"),
        }
    }
    
    pub fn round_to_nearest_multiple_of(value: u64, multiple_of: u64) -> u64 {
        ((value as f64 / multiple_of as f64).round() as u64) * multiple_of
    }
}

mod tests {
    use crate::damselfly::memory::utility::Utility;

    #[test]
    fn convert_seconds_to_microseconds_test() {
        let time = " 0008.157 s ".to_string();
        assert_eq!(Utility::convert_to_microseconds(&time), 8157000);
    }

    #[test]
    fn convert_milliseconds_to_microseconds_test() {
        let time = "0083.339 ms   ".to_string();
        assert_eq!(Utility::convert_to_microseconds(&time), 83339);
    }

    #[test]
    fn convert_microseconds_to_microseconds_test() {
        let time = " 230 us".to_string();
        assert_eq!(Utility::convert_to_microseconds(&time), 230);
    }
}