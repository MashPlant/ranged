`ranged` is a procedural macro that allows user to define ranged integer types.

# Examples

```rust
use ranged::ranged;

#[ranged(-10..=10)]
struct SmallInt(i8);
```

Now `SmallInt` can only hold `i8` that satisfies `-10 <= x && x <= 10`. Three methods will be automatically added to `SmallInt`:

1. `fn new(v: i8) -> Option<SmallInt>`: return `Some(SmallInt(v))` if the input value is within the range, return `None` otherwise
2. `unsafe fn new_unchecked(v: i8) -> SmallInt`: like `new`, but without range check
3. `fn get(self) -> i8`: retrieve the inner value

# Optional Features

1. "rustc-layout": add attribute `#[rustc_layout_scalar_valid_range_{start, end}]` to the struct. Since it is an unstable attribute, user will need to add `#![feature(rustc_attrs)]` to the crate attributes. This may help reduce the size of the enum type that contains this struct. **Note that it won't work properly on the `SmallInt` example**, because currently rustc performs **unsigned** comparison on this attribute.
2. "assume-hint": add `core::intrinsics::assume` about the range of `self.0` to `get`. Since it is an unstable function, user will need to add `#![feature(core_intrinsics)]` to the crate attributes. This may help compiler do optimizations using range information, e.g., currently rustc is able to optimize `-10 <= s.get() && s.get() <= 10` to `true` if `s` is a `SmallInt` when this feature is enabled.

This two features work independently. It is worth noting that currently rustc is **not** able to do such optimization if only "rustc-layout" is enabled and "assume-hint" is not, though it seems possible to do so.