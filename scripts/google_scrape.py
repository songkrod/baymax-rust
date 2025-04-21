# 📁 scripts/google_scrape.py (ตอนนี้ใช้ DuckDuckGo แทน Google)
import sys
import requests
from bs4 import BeautifulSoup

def search_duckduckgo(query):
    headers = {
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
    }
    url = f"https://html.duckduckgo.com/html/?q={query}"
    res = requests.get(url, headers=headers, timeout=5)
    res.raise_for_status()
    soup = BeautifulSoup(res.text, "html.parser")

    result = soup.select_one("a.result__a")
    snippet_elem = soup.select_one("a.result__snippet") or soup.select_one(".result__snippet")

    title = result.text.strip() if result else "ไม่พบหัวข้อ"
    snippet = snippet_elem.text.strip() if snippet_elem else "ไม่พบคำอธิบาย"

    print(f"{title}⧙{snippet}")

if __name__ == "__main__":
    query = " ".join(sys.argv[1:]).strip()
    if not query:
        print("ERROR: Missing query")
        sys.exit(1)
    try:
        search_duckduckgo(query)
    except Exception as e:
        print(f"ERROR: {e}")
        sys.exit(1)
