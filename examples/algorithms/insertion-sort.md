# insertion-sort

## NAME

`insertion-sort` — sort a sequence by growing a sorted prefix

## SYNOPSIS

```text
for each item from left to right:
    move larger items in the sorted prefix one position right
    insert the item into the resulting gap
```

## DESCRIPTION

Insertion sort divides a sequence into a sorted prefix and an unsorted suffix.
It removes the next value from the suffix, finds that value's position in the
prefix, and shifts larger values to make room for it.

The prefix before index `i` is the algorithm's key invariant: it is sorted at
the start and end of every iteration.

## IMPLEMENTATION

```rust
fn insertion_sort<T: Ord>(values: &mut [T]) {
    for i in 1..values.len() {
        let mut j = i;

        while j > 0 && values[j] < values[j - 1] {
            values.swap(j, j - 1);
            j -= 1;
        }
    }
}
```

This version uses adjacent swaps. An optimized implementation can temporarily
remove the current value and shift the larger elements instead.

## COMPLEXITY

| Case | Time |
|---|---:|
| Best, already sorted | `O(n)` |
| Average | `O(n²)` |
| Worst, reverse sorted | `O(n²)` |
| Additional space | `O(1)` |

## PROPERTIES

- **In-place:** yes
- **Stable:** yes, when equal elements are not moved past one another
- **Adaptive:** yes; nearly sorted inputs require relatively little work
- **Online:** it can maintain sorted order as values arrive

## EXAMPLE

Sorting `[5, 2, 4, 6, 1]` grows these sorted prefixes:

```text
[2, 5]          insert 2
[2, 4, 5]       insert 4
[2, 4, 5, 6]    insert 6
[1, 2, 4, 5, 6] insert 1
```

## WHEN TO USE

Use insertion sort for small sequences or data that is already nearly sorted.
It is also useful as the small-partition finishing step in a hybrid sorting
algorithm.

Avoid it for large, randomly ordered inputs where an `O(n log n)` algorithm is
available.

## PITFALLS

- Moving values rather than references may be expensive for large elements.
- Changing the comparison from `<` to `<=` can destroy stability.
- The quadratic worst case matters even though the implementation is simple.

## SEE ALSO

`selection-sort`, `merge-sort`, `binary-search`, `sorting-stability`
