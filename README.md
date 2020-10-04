# organize_rust_imports

Work around https://github.com/rust-lang/rustfmt/issues/2979 by breaking up
single paragraphs of imports into sections for std, external crates, and the
current crate.

Run the tool from the root of your source directory. It'll overwrite files, so
make sure you're under version control, of course! The tool is super brittle;
check the output manually. It's meant to be a one-time tool to canonicalize a
code-base.
