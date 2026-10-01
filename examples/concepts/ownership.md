# ownership

## NAME

`ownership` — Rust's model for managing values and resources

## DESCRIPTION

Every Rust value has an owner. When the owner leaves its scope, Rust drops the
value. Ownership can move to another binding, or code can temporarily borrow
the value through a reference.

The model provides deterministic resource cleanup without a garbage collector.
It applies to memory as well as resources such as files, sockets, and child
process handles.

## RULES

A useful introductory model is:

1. Each value has one owner.
2. Assigning or passing a non-`Copy` value usually moves ownership.
3. Code may have either one mutable reference or any number of immutable
   references to a value at a given time.
4. A reference cannot outlive the value it borrows.

## EXAMPLES

### Move

```rust
let first = String::from("manual");
let second = first;

println!("{second}");
// `first` can no longer be used because ownership moved.
```

### Immutable borrow

```rust
fn byte_count(text: &str) -> usize {
    text.len()
}

let topic = String::from("ownership");
let count = byte_count(&topic);
println!("{topic}: {count}");
```

### Mutable borrow

```rust
fn add_extension(topic: &mut String) {
    topic.push_str(".md");
}

let mut topic = String::from("ownership");
add_extension(&mut topic);
```

## COPY AND CLONE

Small types such as integers implement `Copy`, so assignment copies their bits
instead of moving the original value. Types such as `String` implement `Clone`,
which performs an explicit potentially expensive duplication:

```rust
let first = String::from("manual");
let second = first.clone();
```

Prefer borrowing when code only needs temporary access. Clone when independent
ownership is actually required.

## COMMON ERRORS

- Using a value after it has moved.
- Mutating a value while an immutable borrow is active.
- Returning a reference to a local value.
- Adding `clone()` only to silence the compiler without deciding who should own
  the data.

## SEE ALSO

`borrowing`, `references`, `lifetimes`, `drop`, `copy`, `clone`
