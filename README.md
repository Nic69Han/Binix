# Binix

Binix is a small, dependency-free command-line toolkit for inspecting binary files.

## Install

```sh
pip install .
```

## Usage

```sh
binix info FILE                    # size, detected file type, Shannon entropy
binix hex FILE [-s OFF] [-n LEN] [-w WIDTH]   # hexdump (offsets accept 0x..)
binix strings FILE [-m MIN] [-t]   # printable ASCII strings, -t shows offsets
```

Example:

```text
$ binix info /bin/ls
file:    /bin/ls
size:    142312 bytes
type:    ELF executable
entropy: 5.9349 bits/byte
```

Detected types: ELF, PE, Mach-O, PNG, JPEG, GIF, PDF, ZIP, gzip, bzip2, xz, 7z, Zstandard, WebAssembly, SQLite, tar.

High entropy (close to 8.0) usually means compressed or encrypted content.

## Development

```sh
pip install pytest
python -m pytest
```

## Status

- [x] License (GNU GPL v3)
- [x] Azure Pipelines CI skeleton (runs on `main` and on pull requests)
- [x] `info`, `hex`, `strings` commands
- [x] Tests wired into CI
- [ ] Section/header parsing for ELF and PE
- [ ] Binary diff

## Contributing

1. Create a branch from `main`.
2. Open a pull request; CI runs automatically on it.

## License

Distributed under the GNU General Public License v3.0. See [LICENSE](LICENSE).
