#!/usr/bin/env python3
"""
transcribe.py

แปลงเสียง .wav ให้เป็นข้อความภาษาไทย โดยใช้ faster-whisper
เหมาะสำหรับเรียกจาก Rust ผ่าน Command (stdin/stdout)
"""

import sys
import os
from faster_whisper import WhisperModel

model_size = "tiny"
model = WhisperModel(model_size, device="cpu", compute_type="int8")

def transcribe(file_path):
    segments, _ = model.transcribe(file_path)
    return "".join([s.text for s in segments]).strip()

def main():
    print("[PY] Ready to receive audio path...", flush=True)
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        print(f"[PY] Received path: {line}", flush=True)

        if not os.path.exists(line):
            print("ERROR: File not found", flush=True)
            continue

        try:
            text = transcribe(line)
            print(f"TRANSCRIPT: {text}", flush=True)
        except Exception as e:
            print(f"ERROR: {e}", flush=True)

if __name__ == "__main__":
    main()