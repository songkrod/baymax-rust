# 📁 vector_db.py
from annoy import AnnoyIndex
import os

class VectorDB:
    def __init__(self, index_path: str, dim: int):
        self.index_path = index_path
        self.dim = dim
        self.index = AnnoyIndex(dim, 'angular')
        self.meta = {}  # ใช้เก็บข้อความอธิบาย vector

        # ถ้ามีไฟล์ index แล้ว โหลดเลย
        if os.path.exists(index_path):
            self.index.load(index_path)
            print(f"✅ Index loaded from {index_path}")

    def add(self, vector, meta):
        i = len(self.meta)
        self.index.add_item(i, vector)
        self.meta[i] = meta

    def build_and_save(self, n_trees=10):
        self.index.build(n_trees)
        self.index.save(self.index_path)
        print(f"💾 Index saved to {self.index_path}")

    def search(self, vector, top_n=5):
        return self.index.get_nns_by_vector(vector, top_n)

    def get_meta(self, i):
        return self.meta.get(i, None)
