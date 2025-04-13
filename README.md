# 🤖 Baymax Rust Edition (v3)

Baymax เป็น AI ผู้ช่วยที่พูดได้ ฟังได้ คิดได้  
ทำงานแบบ hybrid: Rust + Python 3.9 + Whisper + GPT + TTS  
ออกแบบให้รันได้ทั้งในเครื่อง dev และ Raspberry Pi Zero 2 W

---

## 🚀 วิธีติดตั้ง (Step-by-step)

### ✅ 1. อัปเดตเครื่องและติดตั้ง dependency ทั้งหมด

```bash
sudo apt update
sudo apt install -y \
  lame \
  espeak \
  portaudio19-dev \
  libasound2-dev \
  libssl-dev \
  libffi-dev \
  libstdc++-12-dev \
  python3.9 python3.9-venv python3.9-dev \
  build-essential
```

---

### ✅ 2. ติดตั้ง Python environment สำหรับ Whisper (ASR)

```bash
python3.9 -m venv venv
source venv/bin/activate
pip install --upgrade pip setuptools wheel
pip install faster-whisper annoy
```

---

### ✅ 3. ทดสอบว่า whisper ใช้ได้ไหม

```bash
source venv/bin/activate
python scripts/transcribe.py /tmp/baymax_audio.wav
```

> ถ้าแสดงข้อความที่แปลงจากเสียง = ใช้งานได้แล้ว

---

### ✅ 4. ตั้งค่า `.env` ที่ root project

```env
tts_backend=google
google_application_credentials=secrets/gcloud_key.json

log_level=info
log_dir=logs
log_file_name=baymax
log_rotate_size=5000000
log_rotate_count=5
```

> ถ้าจะใช้ offline mode (espeak), เปลี่ยน `tts_backend=local`

---

### ✅ 5. เตรียม Google TTS

```bash
mkdir -p secrets
mv key.json secrets/gcloud_key.json
```

> คีย์นี้ได้จาก Google Cloud → Text-to-Speech API → Service Account

---

### ✅ 6. เริ่มรัน Baymax 🎙️

```bash
cargo run
```

> ถ้าทุกอย่างพร้อม Baymax จะพูดว่า  
> `สวัสดีครับ ผมคือเบย์แมกซ์ พร้อมให้บริการครับ`

---

## 📁 โครงสร้างไฟล์หลัก

```
baymax-rust/
├── src/
├── secrets/
│   └── gcloud_key.json
├── scripts/
│   └── transcribe.py
├── venv/
├── .env
├── README.md
```

---

## ❤️ ทีมงาน

## Powered by Rust, GPT, faster-whisper, and a lot of coffee.

## 🧠 Project Structure (Baymax V3)

โครงสร้างระบบ Baymax V3 ออกแบบตามแนวคิด modular และ pluggable

```
src/
├── agent/                 # จุดควบคุมหลักของ AI agent
├── reasoner/              # วิเคราะห์ intent / emotion / response
├── skills/                # สิ่งที่ Baymax ทำได้ เช่น พูด ฟัง ค้นหา
├── services/              # Pluggable adapters เช่น GPT, Whisper, TTS
├── hardware/              # ควบคุมอุปกรณ์จริง เช่น ลำโพง GPIO
├── memory/                # จัดการความจำระยะสั้น-ยาว
├── perception/            # ตรวจจับสิ่งเร้า เช่น เรียกชื่อ
├── utils/                 # ฟังก์ชันทั่วไป เช่น config, logger, vad
├── data/                  # ข้อมูล runtime เช่น log, memory, audio
└── main.rs                # จุดเริ่มต้นโปรแกรม
```

---

## 🧪 Project Instructions for ChatGPT

Baymax Rust Edition ใช้ ChatGPT เป็น reasoning layer แบบ plug-in

### หลักการ:

- **LLM จะไม่ตัดสินใจใช้ hardware หรือเรียก API เอง** แต่ส่ง intent + reply
- GPT ใช้ 2 prompt แยกกัน: insight และ response
- ใช้ symbol `⧙` แทนการแบ่งประโยคใน streaming mode
- Baymax จะพูด "static phrase" ระหว่างรอ GPT เพื่อไม่ให้เงียบเกิน

### แนวทาง prompt:

- `insight_prompt`: วิเคราะห์ emotion, intent, follow_up โดยไม่ stream
- `streaming_prompt`: ใช้สำหรับพูดตอบแบบ real-time, stream ทีละประโยค

### ตัวอย่าง Prompt Instruction:

```text
คุณคือ AI ผู้ช่วยชื่อเบย์แมกซ์ พูดจาอบอุ่น สุภาพ และติดตลกนิดๆ
ห้ามตอบเกินความรู้ที่มี และถ้าตอบไม่ได้ ให้บอกว่า "ขอผมค้นหาข้อมูลสักครู่นะครับ"
แสดงผลลัพธ์สำหรับพูดได้เลย โดยแบ่งประโยคด้วย ⧙
```

---
