// Renders the GhostNote app mark to a 1024x1024 PNG with no image dependencies.
// Run `node scripts/generate-icon.mjs && npx tauri icon icons/source.png` to
// regenerate the full platform icon set.

import { deflateSync } from "node:zlib";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SIZE = 1024;
const OUT = resolve(dirname(fileURLToPath(import.meta.url)), "../icons/source.png");

const BACKDROP = [14, 14, 18];
const PAGE = [244, 244, 245];
const RULE = [113, 113, 122];

/** Signed distance to a rounded rectangle centred at (cx, cy). */
function roundedRectDistance(x, y, cx, cy, halfW, halfH, radius) {
  const dx = Math.abs(x - cx) - (halfW - radius);
  const dy = Math.abs(y - cy) - (halfH - radius);
  const outside = Math.hypot(Math.max(dx, 0), Math.max(dy, 0));
  return outside + Math.min(Math.max(dx, dy), 0) - radius;
}

/** Converts a signed distance to antialiased coverage in [0, 1]. */
function coverage(distance) {
  return Math.min(Math.max(0.5 - distance, 0), 1);
}

function blend(dst, src, alpha) {
  for (let i = 0; i < 3; i += 1) {
    dst[i] = Math.round(dst[i] * (1 - alpha) + src[i] * alpha);
  }
}

// Raw RGBA scanlines, each prefixed with a zero filter byte.
const raw = Buffer.alloc(SIZE * (SIZE * 4 + 1));

for (let y = 0; y < SIZE; y += 1) {
  const rowStart = y * (SIZE * 4 + 1);
  raw[rowStart] = 0;

  for (let x = 0; x < SIZE; x += 1) {
    const px = x + 0.5;
    const py = y + 0.5;

    const backdropAlpha = coverage(
      roundedRectDistance(px, py, SIZE / 2, SIZE / 2, SIZE / 2, SIZE / 2, 232),
    );
    if (backdropAlpha <= 0) continue;

    const color = [...BACKDROP];

    // The note page, held at partial opacity so the mark reads as a "ghost".
    const pageAlpha =
      coverage(roundedRectDistance(px, py, SIZE / 2, SIZE / 2 - 8, 224, 280, 48)) * 0.92;
    blend(color, PAGE, pageAlpha);

    // Three ruled lines, the last one short like an unfinished note.
    for (const [index, halfWidth] of [148, 148, 92].entries()) {
      const lineY = SIZE / 2 - 96 + index * 104;
      const lineAlpha = coverage(
        roundedRectDistance(px, py, SIZE / 2 - (148 - halfWidth), lineY, halfWidth, 17, 17),
      );
      blend(color, RULE, lineAlpha * pageAlpha);
    }

    const offset = rowStart + 1 + x * 4;
    raw[offset] = color[0];
    raw[offset + 1] = color[1];
    raw[offset + 2] = color[2];
    raw[offset + 3] = Math.round(backdropAlpha * 255);
  }
}

function chunk(type, data) {
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length);
  const body = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body) >>> 0);
  return Buffer.concat([length, body, crc]);
}

const CRC_TABLE = Array.from({ length: 256 }, (_, n) => {
  let c = n;
  for (let k = 0; k < 8; k += 1) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  return c >>> 0;
});

function crc32(buf) {
  let c = 0xffffffff;
  for (const byte of buf) c = CRC_TABLE[(c ^ byte) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(SIZE, 0);
ihdr.writeUInt32BE(SIZE, 4);
ihdr[8] = 8; // bit depth
ihdr[9] = 6; // truecolour with alpha
ihdr[10] = 0;
ihdr[11] = 0;
ihdr[12] = 0;

const png = Buffer.concat([
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
  chunk("IHDR", ihdr),
  chunk("IDAT", deflateSync(raw, { level: 9 })),
  chunk("IEND", Buffer.alloc(0)),
]);

mkdirSync(dirname(OUT), { recursive: true });
writeFileSync(OUT, png);
console.log(`wrote ${OUT} (${png.length} bytes)`);
