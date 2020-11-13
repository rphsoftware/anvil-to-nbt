# anvil-to-nbt
A tiny program that extracts all (non-zero length) chunks stored in a given mcregion (mcr) or anvil (mca) file into a directory.

# Compiling
`cargo build --release`

Your binaries should appear in target/release/dump_chunks

# Usage
`./dump_chunks <source_file.mca> <target directory>`

The directory must exist
