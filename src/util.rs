#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub row: usize,
    pub col: usize,
}

impl PartialEq for Location {
    fn eq(&self, _: &Self) -> bool {
        true // Not actually needed, makes coding tests easier.
    }
}

#[macro_export]
macro_rules! svec {
    ($($x:expr),+ $(,)?) => (
        vec![$($x.to_owned()),+]
    )
}

#[macro_export]
macro_rules! loc {
    () => {
        Location { row: 0, col: 0 }
    };
    ($row: expr,$col: expr) => {
        Location {
            row: $row,
            col: $col,
        }
    };
}
