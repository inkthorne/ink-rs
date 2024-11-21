use std::fmt::Display;

// ===========================================================================
// ** Format **
// ===========================================================================

pub struct Format;

impl Format {
    // ---------------------------------------------------------------------------

    pub fn commas1(n: u64) -> String {
        let n_string = format!("{n}");
        let n_length = n_string.len();
        let comma_count = n_length % 3;

        if comma_count == 0 {
            return n_string;
        }

        let mut n_with_commas = String::with_capacity(n_length + comma_count);
        let mut count = 1;

        for c in n_string.chars().rev() {
            n_with_commas.push(c);

            if count % 3 == 0 {
                n_with_commas.push(',');
            }

            count += 1;
        }

        n_with_commas.chars().rev().collect()
    }

    // ---------------------------------------------------------------------------

    pub fn commas<T: Display>(t: T) -> String {
        let t_string = format!("{}", t);
        let t_length = t_string.len();
        let comma_count = (t_length - 1) / 3;

        if comma_count == 0 {
            return t_string;
        }

        let mut t_with_commas = String::with_capacity(t_length + comma_count);
        let mut count = 0;

        for c in t_string.chars().rev() {
            if count % 3 == 0 && count != 0 {
                if c != '-' {
                    t_with_commas.push(',');
                }
            }

            t_with_commas.push(c);
            count += 1;
        }

        t_with_commas.chars().rev().collect()
    }
}

// ===========================================================================
// ** TESTS **
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------

    #[test]
    fn commas() {
        assert_eq!(Format::commas(0), "0");
        assert_eq!(Format::commas(100), "100");
        assert_eq!(Format::commas(1895), "1,895");
        assert_eq!(Format::commas(10000), "10,000");
        assert_eq!(Format::commas(-10000), "-10,000");
        assert_eq!(Format::commas(-100000), "-100,000");
        assert_eq!(Format::commas(-1000000), "-1,000,000");
        assert_eq!(Format::commas(1000000), "1,000,000");
    }
}
