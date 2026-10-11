"""Header and section parsing for ELF and PE executables."""

import struct
from dataclasses import dataclass, field
from typing import Dict, List, Optional


class FormatError(ValueError):
    """Raised when a file claims a format but its headers are malformed."""


@dataclass
class Section:
    name: str
    offset: int
    size: int
    address: int


@dataclass
class Headers:
    format: str
    fields: Dict[str, str]
    sections: List[Section] = field(default_factory=list)


def _unpack(fmt: str, data: bytes, offset: int) -> tuple:
    size = struct.calcsize(fmt)
    if offset < 0 or offset + size > len(data):
        raise FormatError(f"truncated header at offset {offset:#x}")
    return struct.unpack_from(fmt, data, offset)


# ---------------------------------------------------------------- ELF

ELF_TYPES = {0: "NONE", 1: "REL", 2: "EXEC", 3: "DYN", 4: "CORE"}
ELF_MACHINES = {
    0x03: "x86", 0x08: "MIPS", 0x14: "PowerPC", 0x15: "PowerPC64",
    0x28: "ARM", 0x2A: "SuperH", 0x32: "IA-64", 0x3E: "x86-64",
    0xB7: "AArch64", 0xF3: "RISC-V", 0x102: "LoongArch",
}


def parse_elf(data: bytes) -> Headers:
    if data[:4] != b"\x7fELF":
        raise FormatError("not an ELF file")
    if len(data) < 16:
        raise FormatError("truncated ELF identification")
    ei_class, ei_data = data[4], data[5]
    if ei_class not in (1, 2):
        raise FormatError(f"invalid ELF class {ei_class}")
    if ei_data not in (1, 2):
        raise FormatError(f"invalid ELF data encoding {ei_data}")
    e = "<" if ei_data == 1 else ">"
    is64 = ei_class == 2

    if is64:
        (e_type, e_machine, _, e_entry, _, e_shoff, _, _, _, _,
         e_shentsize, e_shnum, e_shstrndx) = _unpack(e + "HHIQQQIHHHHHH", data, 16)
    else:
        (e_type, e_machine, _, e_entry, _, e_shoff, _, _, _, _,
         e_shentsize, e_shnum, e_shstrndx) = _unpack(e + "HHIIIIIHHHHHH", data, 16)

    fields = {
        "class": "ELF64" if is64 else "ELF32",
        "endianness": "little" if ei_data == 1 else "big",
        "type": ELF_TYPES.get(e_type, f"{e_type:#x}"),
        "machine": ELF_MACHINES.get(e_machine, f"{e_machine:#x}"),
        "entry": f"{e_entry:#x}",
        "sections": str(e_shnum),
    }

    sections: List[Section] = []
    if e_shoff and e_shnum:
        shfmt = e + ("IIQQQQIIQQ" if is64 else "IIIIIIIIII")
        if e_shentsize < struct.calcsize(shfmt):
            raise FormatError(f"section header entry too small ({e_shentsize})")
        raw = []
        for i in range(e_shnum):
            sh = _unpack(shfmt, data, e_shoff + i * e_shentsize)
            # sh_name, sh_type, sh_flags, sh_addr, sh_offset, sh_size, ...
            raw.append((sh[0], sh[3], sh[4], sh[5]))
        strtab = b""
        if e_shstrndx < len(raw):
            _, _, off, size = raw[e_shstrndx]
            strtab = data[off:off + size]
        for name_off, addr, off, size in raw:
            end = strtab.find(b"\x00", name_off)
            name = strtab[name_off:end if end >= 0 else None].decode("ascii", "replace")
            sections.append(Section(name, off, size, addr))
    return Headers("ELF", fields, sections)


# ---------------------------------------------------------------- PE

PE_MACHINES = {
    0x014C: "x86", 0x8664: "x86-64", 0x01C0: "ARM", 0x01C4: "ARMv7 Thumb",
    0xAA64: "ARM64", 0x0200: "IA-64", 0x5064: "RISC-V64",
}
PE_SUBSYSTEMS = {
    1: "native", 2: "Windows GUI", 3: "Windows console", 9: "Windows CE",
    10: "EFI application", 11: "EFI boot driver", 12: "EFI runtime driver",
}


def parse_pe(data: bytes) -> Headers:
    if data[:2] != b"MZ":
        raise FormatError("not an MZ executable")
    (pe_off,) = _unpack("<I", data, 0x3C)
    if data[pe_off:pe_off + 4] != b"PE\x00\x00":
        raise FormatError("missing PE signature (plain DOS executable?)")
    (machine, nsections, timestamp, _, _, opt_size, characteristics) = _unpack(
        "<HHIIIHH", data, pe_off + 4)
    opt = pe_off + 24
    (magic,) = _unpack("<H", data, opt)
    if magic == 0x10B:
        kind, (entry,) = "PE32", _unpack("<I", data, opt + 16)
        (image_base,) = _unpack("<I", data, opt + 28)
        (subsystem,) = _unpack("<H", data, opt + 68)
    elif magic == 0x20B:
        kind, (entry,) = "PE32+", _unpack("<I", data, opt + 16)
        (image_base,) = _unpack("<Q", data, opt + 24)
        (subsystem,) = _unpack("<H", data, opt + 68)
    else:
        raise FormatError(f"unknown optional header magic {magic:#x}")

    fields = {
        "class": kind,
        "machine": PE_MACHINES.get(machine, f"{machine:#x}"),
        "type": "DLL" if characteristics & 0x2000 else "EXE",
        "subsystem": PE_SUBSYSTEMS.get(subsystem, str(subsystem)),
        "image base": f"{image_base:#x}",
        "entry": f"{entry:#x}",
        "timestamp": str(timestamp),
        "sections": str(nsections),
    }

    sections = []
    table = opt + opt_size
    for i in range(nsections):
        name, vsize, vaddr, rawsize, rawptr = _unpack("<8sIIII", data, table + i * 40)
        sections.append(Section(name.rstrip(b"\x00").decode("ascii", "replace"),
                                rawptr, rawsize, image_base + vaddr))
    return Headers(kind, fields, sections)


def parse(data: bytes) -> Optional[Headers]:
    """Parse ELF or PE headers; return None for other formats."""
    if data[:4] == b"\x7fELF":
        return parse_elf(data)
    if data[:2] == b"MZ":
        return parse_pe(data)
    return None
