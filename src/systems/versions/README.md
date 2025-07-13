# Version constraint solving

There's a few key ideas for Version solving:

## Version caching

In order to avoid the repeated fetching that version solving would incur,\
we need to properly cache the fetched versions of packages that are fetched\
and the version constraints that those packages introduce.

## Proper conflict propagation

We want to properly detect when certain versions of packages cause conflicts.\
This requires 2 main things: being able to attach versions of packages being\
installed to the (potential) conflicts that they cause and being able to merge\
the conflicts when the same installations consistently cause issues.

In terms of attaching installed packages to conflicts caused we should probably\
be using a data structure such as

```rust
HashMap<HashSet<(Box<str>, Version)>, Result<(), Conflicts>>
```

where:

- `Box<str>` is the name of the package installed
- `Version` is a type representing the package version
- `Result<(), Conflicts>` is the potential conflicts the package causes
