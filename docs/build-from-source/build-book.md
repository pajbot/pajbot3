# Build this documentation

The documentation is written in a format to be compiled by [mdBook](https://github.com/rust-lang/mdBook). The [prerequisites chapter](prerequisites.md) covers the steps to install the `mdbook` tool.

To build:

```none
$ cd docs
$ mdbook build
```

Static HTML output is generated in `target/book`.

You may also find `mdbook serve` useful while developing the documentation. (Serves the book at http://localhost:3000, and rebuilds it on changes)
