mod position;
mod splash;

pub use splash::{SplashScreen, SplashScreenConfig};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
