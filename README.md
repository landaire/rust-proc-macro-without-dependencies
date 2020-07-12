# rust-proc-macro-without-dependencies

This repo shows an example of how to create a proc macro in rust without any dependencies. The [default_derive](./default_derive) crate provides a proc macro, `OurDefault`, which re-implements the builtin `Default` macro.

## Credits

The logic in this crate was written by referencing [nanoserde](https://github.com/not-fl3/nanoserde/blob/ceab998766086a9ce2ae88b66548622c1d726def/derive/src/parse.rs)'s parsing logic and influenced by my [rants](https://internals.rust-lang.org/t/breakage-of-fragile-proc-macros-in-nightly-2020-07-03/12688) on the rust internals forum.

## Support

The intent is not to cover all possible scenarios, but instead to provide a minimal reference for what I believe is the most common usecase of a simple proc macro: a struct with named fields, no lifetimes, and no generic parameters. It currently does not support any of the following scenarios:

### Enums

```rust
#[derive(OurDefault)]
enum Foo {
    Bar,
    Baz
}
```

### Unit struct

```rust
struct Foo;
```

(ok, this is just me being lazy and not wanting to complicate the example)

### Unnamed structs

```rust
struct Foo(String, usize, u32);
```

### Any visibility modifiers that are not just `pub`

```rust
pub(crate) struct Foo { bar: String }
```

### Lifetimes, references, generic parameters

```rust
pub(crate) struct Foo<'a, T> { bar: &'a T }
```

### Attributes

```rust
#[repr(C)]
pub(crate) struct Foo { bar: String }
```

```rust
#[my_custom_attribute]
pub(crate) struct Foo { bar: String }
```

```rust
pub(crate) struct Foo { #[my_custom_attribute] bar: String }
```

## Output

The `default_derive_tester` test application will simply test to ensure that the result of `TestStruct::our_default()` matches `Default::default()`. Ths emitted tokens look like:


```rust
pub struct TestStruct {
    foo: String,
    pub bar: usize,
}
#[automatically_derived]
impl crate::OurDefault for TestStruct {
    fn our_default() -> Self {
        TestStruct {
            foo: Default::default(),
            bar: Default::default(),
        }
    }
}
```