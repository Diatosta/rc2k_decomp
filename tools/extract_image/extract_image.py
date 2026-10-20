#!/usr/bin/env python3
"""tools/extract_image/extract_image.py RAL.EXE data/ral_image.bin

What the standalone program keeps of RAL.EXE (decoupling, step B; owner 2026-10-19: the
program's own data may be committed): every section that is not code (`.rdata`, `.data`,
`.idata`, `STACK`, `.rsrc`; not `.text`, `GURUS`, `.reloc`), packed, and the relocations
that fall in them; of the code sections only the data IDA marks in them
(`code_data.txt`: the log's month and day names, jump tables and the like), no code. `standalone.rs` builds the
image from it at the same places, so that every hook's game addresses hold.

The file (little-endian):
  "RC2KIMG1", image size, the base the file wants, section count;
  per section: its rva, virtual size, flags (1 code, 2 kept, 4 pieces), and for a kept one
  its data as runs: (zeros, literal length, literal bytes) .. until the section's raw size;
  for one with pieces: their count, then each piece's rva, length and bytes;
  relocation count, then each relocation's rva (a dword that holds an address);
  "END!".
"""
import struct, sys, hashlib

def main(exe, out):
    b = open(exe, "rb").read()
    import os
    pieces = []
    for line in open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "code_data.txt")):
        line = line.strip()
        if line and not line.startswith("#"):
            start, end = (int(x, 16) for x in line.split())
            pieces.append((start, end))
    pe = struct.unpack_from("<I", b, 0x3C)[0]
    assert b[pe:pe+4] == b"PE\0\0", "not a program file"
    count = struct.unpack_from("<H", b, pe + 6)[0]
    opt_len = struct.unpack_from("<H", b, pe + 0x14)[0]
    opt = pe + 0x18
    base = struct.unpack_from("<I", b, opt + 0x1C)[0]
    size = struct.unpack_from("<I", b, opt + 0x38)[0]
    reloc_rva, reloc_len = struct.unpack_from("<II", b, opt + 0x60 + 5 * 8)
    sections = []
    for n in range(count):
        s = opt + opt_len + 40 * n
        name = b[s:s+8].rstrip(b"\0").decode()
        vlen, rva, rlen, raw, = struct.unpack_from("<IIII", b, s + 8)
        flags = struct.unpack_from("<I", b, s + 36)[0]
        code = flags & 0x20000020 != 0
        keep = not code and name != ".reloc"
        sections.append((name, rva, vlen, rlen, raw, code, keep))
    # the relocations, from the file's .reloc
    reloc_sec = next(x for x in sections if x[0] == ".reloc")
    rel = b[reloc_sec[4]:reloc_sec[4] + reloc_len]
    places = []
    at = 0
    while at + 8 <= len(rel):
        page, blen = struct.unpack_from("<II", rel, at)
        if blen < 8:
            break
        for k in range((blen - 8) // 2):
            e = struct.unpack_from("<H", rel, at + 8 + 2 * k)[0]
            if e >> 12 == 3:
                places.append(page + (e & 0xFFF))
            elif e >> 12 != 0:
                raise SystemExit(f"a relocation of kind {e >> 12}")
        at += blen
    kept = [s for s in sections if s[6]]
    def in_kept(r):
        return any(s[1] <= r and r + 4 <= s[1] + max(s[2], s[3]) for s in kept) or any(
            a - base <= r and r + 4 <= e - base for a, e in pieces)
    places = [p for p in places if in_kept(p)]
    o = bytearray(b"RC2KIMG1")
    o += struct.pack("<III", size, base, count)
    for name, rva, vlen, rlen, raw, code, keep in sections:
        mine = [(a - base, e - base) for a, e in pieces if rva <= a - base < rva + vlen]
        flags = (1 if code else 0) | (2 if keep else 0) | (4 if code and mine else 0)
        o += struct.pack("<III", rva, vlen, flags)
        if code and mine:
            o += struct.pack("<I", len(mine))
            for a, e in mine:
                o += struct.pack("<II", a, e - a) + b[raw + a - rva:raw + e - rva]
        if not keep:
            continue
        data = b[raw:raw + rlen]
        o += struct.pack("<I", len(data))
        i = 0
        while i < len(data):
            z = i
            while z < len(data) and data[z] == 0:
                z += 1
            lit = z
            # a literal run ends at 16 zeros in a row (a shorter gap is cheaper kept)
            while lit < len(data):
                if data[lit] == 0 and data[lit:lit+16] == b"\0" * min(16, len(data) - lit):
                    break
                lit += 1
            o += struct.pack("<II", z - i, lit - z) + data[z:lit]
            i = lit
    o += struct.pack("<I", len(places))
    for p in places:
        o += struct.pack("<I", p)
    o += b"END!"
    open(out, "wb").write(o)
    print(f"{out}: {len(o)} bytes; sections kept {[s[0] for s in kept]}; "
          f"{len(places)} relocations; RAL.EXE sha1 {hashlib.sha1(b).hexdigest()}")

if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
