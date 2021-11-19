# Build this documentation

The documentation is written in a format to be compiled by [mdBook](https://github.com/rust-lang/mdBook). The [prerequisites chapter](prerequisites.md) covers the steps to install the `mdbook` tool.

To build, use:

```none
cd docs && mdbook build
```

or

```none
mdbook build docs
```

Static HTML output is generated in `target/book`.

You may also find `mdbook serve` useful while developing the documentation. (Serves the book at http://localhost:3000, and rebuilds it on changes)

Other `mdbook` commands are documented via `mdbook --help`.

Similarly to how you can either `cd docs` or add `docs` to the `build` command, the same applies for most other `mdbook` commands, including `serve`.
