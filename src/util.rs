#[macro_export]
macro_rules! svec {
    ($($x:expr),+ $(,)?) => (
        vec![$($x.to_owned()),+]
    )
}
