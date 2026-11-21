#!/usr/bin/env python3
"""Regenerate tests/fixtures/sample.wav (synthetic, non-PII)."""
import math
import struct
import wave

SR = 16_000
samples = []
for duration, voiced in [(1.2, True), (0.6, False), (1.2, True)]:
    for index in range(int(SR * duration)):
        if voiced:
            envelope = 0.15 + 0.15 * math.sin(2 * math.pi * 3 * index / SR)
            samples.append(envelope * math.sin(2 * math.pi * 180 * index / SR))
        else:
            samples.append(0.0)

with wave.open("tests/fixtures/sample.wav", "wb") as handle:
    handle.setnchannels(1)
    handle.setsampwidth(2)
    handle.setframerate(SR)
    handle.writeframes(
        b"".join(struct.pack("<h", int(max(-1.0, min(1.0, s)) * 32767)) for s in samples)
    )
