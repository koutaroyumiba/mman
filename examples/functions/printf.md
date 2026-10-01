# printf

## NAME

`printf` — write formatted output to standard output

## SYNOPSIS

```c
#include <stdio.h>

int printf(const char *restrict format, ...);
```

## DESCRIPTION

`printf` writes text to `stdout` according to a format string. Ordinary
characters are copied directly. A conversion specification beginning with `%`
consumes an additional argument and formats it.

A conversion generally has this form:

```text
%[flags][width][.precision][length]conversion
```

Common conversions:

| Conversion | Expected value | Output |
|---|---|---|
| `%d`, `%i` | `int` | Signed decimal integer |
| `%u` | `unsigned int` | Unsigned decimal integer |
| `%x` | `unsigned int` | Hexadecimal integer |
| `%f` | `double` | Decimal floating-point number |
| `%c` | `int` | Character |
| `%s` | `char *` | Null-terminated string |
| `%p` | `void *` | Pointer representation |
| `%%` | None | Literal `%` |

## PARAMETERS

`format`
: Null-terminated format string describing the output and its arguments.

`...`
: Values consumed by the conversion specifications in `format`.

## RETURN VALUE

Returns the number of characters written, excluding the terminating null byte.
Returns a negative value if an output or encoding error occurs.

## EXAMPLES

Print a string and an integer:

```c
const char *name = "Ada";
int score = 42;

printf("%s scored %d points\n", name, score);
```

Set a minimum width and precision:

```c
printf("total: %8.2f\n", 12.5);
```

## ERRORS AND PITFALLS

- A conversion and its corresponding argument must have compatible types;
  otherwise behavior is undefined.
- User-controlled text should be passed as data, not as the format string:

  ```c
  printf("%s", user_input);  /* safe form */
  ```

- `printf` may buffer output. A newline does not guarantee flushing in every
  context.
- Check the return value when output failure matters.

## SEE ALSO

`fprintf`, `snprintf`, `puts`, `stdout`, `format-string`
