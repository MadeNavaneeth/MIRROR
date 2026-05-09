#!/usr/bin/env python3
"""
Build UI/UX Knowledge Base from PDF library
Extracts text, chunks it, and prepares for embedding
"""

import os
import json
from pathlib import Path
from typing import List, Dict
import sys

try:
    import pdfplumber
except ImportError:
    print("ERROR: pdfplumber not installed. Run: pip3 install pdfplumber")
    sys.exit(1)

# Configuration
LIBRARY_PATH = "ui-ux-library"
CHUNK_SIZE = 1000
CHUNK_OVERLAP = 200
OUTPUT_DIR = "uiux_knowledge_base"

CATEGORIES = [
    "Introductory Books",
    "User Experience",
    "User Interface",
    "Interaction Design",
    "Mobile Design",
    "Web Design",
    "Wireframes, Mockups, Prototypes",
    "Style Guides",
    "Typography",
    "Usability Testing",
    "Marketing & Conversion",
    "Collaboration"
]

def simple_chunk_text(text: str, chunk_size: int = CHUNK_SIZE, overlap: int = CHUNK_OVERLAP) -> List[str]:
    """Simple text chunking with overlap"""
    chunks = []
    start = 0
    text_len = len(text)
    
    while start < text_len:
        end = start + chunk_size
        chunk = text[start:end]
        
        # Try to break at sentence boundary
        if end < text_len:
            last_period = chunk.rfind('. ')
            if last_period > chunk_size // 2:
                end = start + last_period + 2
                chunk = text[start:end]
        
        if chunk.strip():
            chunks.append(chunk.strip())
        
        start = end - overlap
    
    return chunks

def extract_pdf_text(pdf_path: Path) -> List[Dict]:
    """Extract text from PDF with page numbers"""
    chunks = []
    try:
        with pdfplumber.open(pdf_path) as pdf:
            for page_num, page in enumerate(pdf.pages, 1):
                text = page.extract_text()
                if text and text.strip():
                    chunks.append({
                        "text": text.strip(),
                        "page": page_num
                    })
    except Exception as e:
        print(f"  ⚠️  Error extracting {pdf_path.name}: {e}")
    return chunks

def process_library():
    """Process all PDFs in the library"""
    output_path = Path(OUTPUT_DIR)
    output_path.mkdir(exist_ok=True)
    
    all_chunks = []
    total_pdfs = 0
    
    for category in CATEGORIES:
        category_path = Path(LIBRARY_PATH) / category
        if not category_path.exists():
            print(f"⚠️  Category not found: {category}")
            continue
            
        print(f"\n📚 Processing category: {category}")
        
        pdf_files = list(category_path.glob("*.pdf"))
        if not pdf_files:
            print(f"  No PDFs found")
            continue
        
        for pdf_file in pdf_files:
            print(f"  📄 {pdf_file.name[:50]}...")
            total_pdfs += 1
            
            # Extract pages
            pages = extract_pdf_text(pdf_file)
            
            if not pages:
                print(f"     ⚠️  No text extracted")
                continue
            
            # Process each page
            for page_data in pages:
                # Chunk the page text
                text_chunks = simple_chunk_text(page_data["text"])
                
                for chunk_idx, chunk_text in enumerate(text_chunks):
                    chunk_id = f"uiux_{category.replace(' ', '_').replace(',', '').replace('&', 'and')}_{pdf_file.stem.replace(' ', '_')}_{page_data['page']}_{chunk_idx}"
                    
                    all_chunks.append({
                        "id": chunk_id,
                        "text": chunk_text,
                        "source": pdf_file.stem,
                        "category": category,
                        "page": page_data["page"],
                        "chunk_index": chunk_idx
                    })
            
            print(f"     ✅ {len(pages)} pages, {sum(len(simple_chunk_text(p['text'])) for p in pages)} chunks")
    
    # Save chunks
    output_file = output_path / "chunks.jsonl"
    with open(output_file, 'w', encoding='utf-8') as f:
        for chunk in all_chunks:
            f.write(json.dumps(chunk, ensure_ascii=False) + '\n')
    
    print(f"\n{'='*60}")
    print(f"✅ EXTRACTION COMPLETE")
    print(f"{'='*60}")
    print(f"📊 PDFs processed: {total_pdfs}")
    print(f"📊 Total chunks: {len(all_chunks)}")
    print(f"📁 Output: {output_file}")
    print(f"{'='*60}\n")
    
    return all_chunks

if __name__ == "__main__":
    print("🚀 Starting UI/UX Knowledge Base Extraction\n")
    chunks = process_library()
