# 📁 test_vector_db.py
from vector_db import VectorDB

def test_vector_db():
    db = VectorDB("src/data/memory/index.ann", dim=5)

    # เพิ่มตัวอย่าง vector พร้อมข้อความ
    db.add([0.1, 0.2, 0.3, 0.4, 0.5], "user: หิวข้าว")
    db.add([0.2, 0.3, 0.4, 0.5, 0.6], "bot: ผมหาข้าวให้ได้นะครับ")
    db.build_and_save()

    # ลองค้นหา
    result = db.search([0.1, 0.2, 0.3, 0.4, 0.5])
    print("🔍 Result indices:", result)
    for r in result:
        print("→", db.get_meta(r))

if __name__ == "__main__":
    test_vector_db()