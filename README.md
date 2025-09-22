# nice_trie

Compressed radix trie implementation in rust (also known as compact prefix tree)

> This is work in progress and is not properly tested or benchmarked.

## Features

- Polymorphic on radix algorithm, choose whichever is best for your use case or write your own.
- Stores nodes in a single vector for better cache locality.
- Only allocates each edge segment once.
- Fully generic in terms of keys and values.
- Supports update operations.
- Efficiently converts between external/internal key representations.

## Speculative comparison (why I decided to write my own)

#### radix_trie

Might be the most popular rust trie crate.

- Each node is independently heap allocated.
- Stores the entire key for each node.
- Values are always boxed, rather than leaving this choice to the user.
- Claims no unsafe code, but uses `smallvec`, which is full of `unsafe`.
- Uses an algorithm with 4-bit nibbles and max branching factor of 16 (supposedly optimal for many use cases).

#### trie-hard

Clouflare's implementation optimized for read-only use with high miss rates.

- Uses a very smart binary mask algorithm to save on routing.
- Only supports bulk construction, no update operations.
