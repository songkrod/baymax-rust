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

Powered by Rust, GPT, faster-whisper, and a lot of coffee.
