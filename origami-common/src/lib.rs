pub mod lockfile;

#[cfg(install)]
pub const LIBEXECDIR: &str = env!("ORI_BUILD_LIBEXECDIR");

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
