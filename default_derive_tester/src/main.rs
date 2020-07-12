use default_derive::OurDefault;

trait OurDefault {
    fn our_default() -> Self;
}

fn main() {
    #[derive(Debug, PartialEq, Eq, OurDefault, Default)]
    pub struct TestStruct {
        foo: String,
        pub bar: usize,
    }

    assert_eq!(TestStruct::our_default(), TestStruct::default())
}
