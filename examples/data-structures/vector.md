# vector

## NAME

`vector` — growable, contiguous sequence of values

## SYNOPSIS

```rust
let mut values: Vec<i32> = Vec::new();
values.push(10);
values.push(20);
```

## DESCRIPTION

A vector stores elements contiguously and can change its length at runtime. It
usually owns a heap allocation described by three values:

- a pointer to the allocation,
- the number of initialized elements (`length`), and
- the number of elements that fit before reallocation (`capacity`).

The invariant is:

```text
length <= capacity
```

When insertion exceeds capacity, the vector allocates a larger region and moves
its elements. The growth strategy makes repeated append operations amortized
constant time.

## OPERATIONS

| Operation | Typical complexity |
|---|---:|
| Index by position | `O(1)` |
| Read or replace the last element | `O(1)` |
| Append | Amortized `O(1)` |
| Remove the last element | `O(1)` |
| Insert or remove near the front | `O(n)` |
| Linear search | `O(n)` |

## RUST INTERFACE

```rust
let mut names = Vec::with_capacity(4);

names.push(String::from("Ada"));
names.push(String::from("Grace"));

let first = names.get(0);       // Option<&String>
let last = names.pop();         // Option<String>
let count = names.len();
let reserved = names.capacity();
```

Use `get` when an index may be invalid. Indexing with `names[index]` panics when
the index is out of bounds.

## MEMORY AND INVALIDATION

Appending may reallocate the backing storage. Reallocation changes the address
of every element. Rust's borrowing rules prevent a safe program from retaining
an element reference while mutating the vector in a way that could reallocate.

`Vec<T>` owns its elements. Dropping the vector drops its initialized elements
and then releases its allocation.

## WHEN TO USE

Use a vector when:

- order matters,
- indexed access is common,
- iteration speed matters, or
- most insertions and removals occur at the end.

Consider another structure when frequent insertion at the front, stable element
addresses, or lookup by key is the dominant requirement.

## PITFALLS

- Confusing length with capacity.
- Indexing without validating external input.
- Repeatedly inserting at index zero, which shifts all existing elements.
- Assuming capacity grows by a particular factor; that is an implementation
  detail.

## SEE ALSO

`array`, `slice`, `deque`, `linked-list`, `hash-map`
