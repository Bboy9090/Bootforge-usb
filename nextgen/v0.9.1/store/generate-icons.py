from pathlib import Path
import binascii
import struct
import zlib

root = Path(__file__).resolve().parent
icon_dir = root / "icons"
icon_dir.mkdir(parents=True, exist_ok=True)

def chunk(kind: bytes, payload: bytes) -> bytes:
    return (
        struct.pack(">I", len(payload))
        + kind
        + payload
        + struct.pack(">I", binascii.crc32(kind + payload) & 0xFFFFFFFF)
    )

width = height = 32
rows = []
for y in range(height):
    row = bytearray([0])
    for x in range(width):
        edge = x < 3 or y < 3 or x >= width - 3 or y >= height - 3
        row.extend((24, 32, 44, 255) if edge else (52, 120, 246, 255))
    rows.append(bytes(row))

rgba = b"".join(rows)
png = (
    b"\x89PNG\r\n\x1a\n"
    + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
    + chunk(b"IDAT", zlib.compress(rgba, 9))
    + chunk(b"IEND", b"")
)
(icon_dir / "icon.png").write_bytes(png)
ico = (
    struct.pack("<HHH", 0, 1, 1)
    + struct.pack("<BBBBHHII", width, height, 0, 0, 1, 32, len(png), 22)
    + png
)
(icon_dir / "icon.ico").write_bytes(ico)
