import struct

import pytest

from binix.cli import main
from binix.formats import FormatError, parse, parse_elf, parse_pe


def make_elf(is64=True, little=True, entry=0x401000):
    """Minimal ELF with a null section, .text and .shstrtab."""
    e = "<" if little else ">"
    shstrtab = b"\x00.text\x00.shstrtab\x00"
    ehsize = 64 if is64 else 52
    shentsize = 64 if is64 else 40
    strtab_off = ehsize
    shoff = strtab_off + len(shstrtab)
    ident = b"\x7fELF" + bytes([2 if is64 else 1, 1 if little else 2, 1]) + b"\x00" * 9
    if is64:
        hdr = struct.pack(e + "HHIQQQIHHHHHH", 2, 0x3E, 1, entry, 0, shoff, 0,
                          ehsize, 0, 0, shentsize, 3, 2)
        sh = lambda n, a, o, s: struct.pack(e + "IIQQQQIIQQ", n, 1, 0, a, o, s, 0, 0, 0, 0)
    else:
        hdr = struct.pack(e + "HHIIIIIHHHHHH", 2, 0x08, 1, entry, 0, shoff, 0,
                          ehsize, 0, 0, shentsize, 3, 2)
        sh = lambda n, a, o, s: struct.pack(e + "IIIIIIIIII", n, 1, 0, a, o, s, 0, 0, 0, 0)
    sections = sh(0, 0, 0, 0) + sh(1, entry, 0, 0x20) + sh(7, 0, strtab_off, len(shstrtab))
    return ident + hdr + shstrtab + sections


def make_pe(dll=False):
    """Minimal PE32+ (x86-64) with one .text section."""
    pe_off = 0x40
    dos = b"MZ" + b"\x00" * 0x3A + struct.pack("<I", pe_off)
    opt_size = 0xF0
    coff = b"PE\x00\x00" + struct.pack("<HHIIIHH", 0x8664, 1, 1700000000, 0, 0,
                                       opt_size, 0x2022 if dll else 0x22)
    opt = bytearray(opt_size)
    struct.pack_into("<H", opt, 0, 0x20B)
    struct.pack_into("<I", opt, 16, 0x1234)
    struct.pack_into("<Q", opt, 24, 0x140000000)
    struct.pack_into("<H", opt, 68, 3)
    sect = struct.pack("<8sIIII", b".text", 0x100, 0x1000, 0x200, 0x400) + b"\x00" * 16
    return dos + coff + bytes(opt) + sect


def test_elf64_little():
    h = parse_elf(make_elf())
    assert h.fields["class"] == "ELF64"
    assert h.fields["machine"] == "x86-64"
    assert h.fields["type"] == "EXEC"
    assert h.fields["entry"] == "0x401000"
    assert [s.name for s in h.sections] == ["", ".text", ".shstrtab"]
    assert h.sections[1].address == 0x401000


def test_elf32_big_endian():
    h = parse_elf(make_elf(is64=False, little=False, entry=0x400))
    assert h.fields["class"] == "ELF32"
    assert h.fields["endianness"] == "big"
    assert h.fields["machine"] == "MIPS"
    assert h.sections[1].name == ".text"


def test_elf_truncated():
    with pytest.raises(FormatError):
        parse_elf(make_elf()[:30])


def test_elf_bad_class():
    data = bytearray(make_elf())
    data[4] = 9
    with pytest.raises(FormatError):
        parse_elf(bytes(data))


def test_pe32_plus():
    h = parse_pe(make_pe())
    assert h.fields["class"] == "PE32+"
    assert h.fields["machine"] == "x86-64"
    assert h.fields["type"] == "EXE"
    assert h.fields["subsystem"] == "Windows console"
    assert h.fields["entry"] == "0x1234"
    assert len(h.sections) == 1
    s = h.sections[0]
    assert (s.name, s.offset, s.size, s.address) == (".text", 0x400, 0x200, 0x140001000)


def test_pe_dll_flag():
    assert parse_pe(make_pe(dll=True)).fields["type"] == "DLL"


def test_plain_dos_executable_rejected():
    with pytest.raises(FormatError):
        parse_pe(b"MZ" + b"\x00" * 0x3E)


def test_parse_dispatch():
    assert parse(make_elf()).format == "ELF"
    assert parse(make_pe()).format == "PE32+"
    assert parse(b"hello") is None


def test_cli_headers(tmp_path, capsys):
    f = tmp_path / "a.exe"
    f.write_bytes(make_pe())
    assert main(["headers", str(f)]) == 0
    out = capsys.readouterr().out
    assert "PE32+" in out and ".text" in out


def test_cli_headers_malformed(tmp_path, capsys):
    f = tmp_path / "bad"
    f.write_bytes(b"\x7fELF\x02\x01\x01")
    assert main(["headers", str(f)]) == 1
    assert "malformed headers" in capsys.readouterr().err


def test_cli_headers_unsupported(tmp_path, capsys):
    f = tmp_path / "x.txt"
    f.write_text("hi")
    assert main(["headers", str(f)]) == 1
    assert "not an ELF or PE" in capsys.readouterr().err
