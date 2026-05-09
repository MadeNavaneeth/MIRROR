#!/usr/bin/env python3
"""
Generate embeddings for UI/UX knowledge chunks using Ollama
and import into ZeroClaw's SQLite memory database
"""

import json
import sqlite3
import subprocess
from pathlib import Path
from typing import List, Dict
import time

# Configuration
CHUNKS_FILE = "uiux_knowledge_base/chunks.jsonl"
ZEROCLAW_DB = Path.home() / ".zeroclaw" / "memory.db"
OLLAMA_MODEL = "nomic-embed-text"
BATCH_SIZE = 100  # Process in batches to avoid memory issues

def get_embedding(text: str) -> List[float]:
    """Generate embedding using Ollama"""
    try:
        result = subprocess.run(
            ["ollama", "embed", OLLAMA_MODEL, text],
            capture_output=True,
            text=True,
            timeout=30
        )
        if result.returncode == 0:
            # Parse the embedding from output
            embedding = json.loads(result.stdout)
            return embedding
        else:
            print(f"Error generating embedding: {result.stderr}")
            return None
    except Exception as e:
        print(f"Error: {e}")
        return None

def import_to_zeroclaw(chunks: List[Dict]):
    """Import chunks with embeddings into ZeroClaw memory database"""
    
    # Connect to ZeroClaw's SQLite database
    conn = sqlite3.connect(ZEROCLAW_DB)
    cursor = conn.cursor()
    
    # Create table if it doesn't exist (matching ZeroClaw's schema)
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS uiux_knowledge (
            id TEXT PRIMARY KEY,
            text TEXT NOT NULL,
            source TEXT,
            category TEXT,
            page INTEGER,
            chunk_index INTEGER,
            embedding BLOB,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
    """)
    
    total = len(chunks)
    imported = 0
    skipped = 0
    
    for i, chunk in enumerate(chunks, 1):
        print(f"\r[{i}/{total}] Generating embedding and importing... ", end="", flush=True)
        
        # Create enriched text with metadata
        enriched_text = f"""Source: {chunk['source']}
Category: {chunk['category']}
Page: {chunk['page']}

{chunk['text']}"""
        
        # Generate embedding
        embedding = get_embedding(chunk['text'])
        
        if embedding is None:
            skipped += 1
            continue
        
        # Convert embedding to bytes for storage
        embedding_bytes = json.dumps(embedding).encode('utf-8')
        
        try:
            # Insert into database
            cursor.execute("""
                INSERT OR REPLACE INTO uiux_knowledge 
                (id, text, source, category, page, chunk_index, embedding)
                VALUES (?, ?, ?, ?, ?, ?, ?)
            """, (
                chunk['id'],
                enriched_text,
                chunk['source'],
                chunk['category'],
                chunk['page'],
                chunk['chunk_index'],
                embedding_bytes
            ))
            imported += 1
            
            # Commit in batches
            if i % BATCH_SIZE == 0:
                conn.commit()
                print(f"\n✅ Batch {i//BATCH_SIZE} committed ({imported} imported, {skipped} skipped)")
        
        except Exception as e:
            print(f"\n⚠️  Error importing chunk {chunk['id']}: {e}")
            skipped += 1
    
    # Final commit
    conn.commit()
    conn.close()
    
    print(f"\n\n{'='*60}")
    print(f"✅ IMPORT COMPLETE")
    print(f"{'='*60}")
    print(f"📊 Total chunks: {total}")
    print(f"📊 Imported: {imported}")
    print(f"📊 Skipped: {skipped}")
    print(f"📁 Database: {ZEROCLAW_DB}")
    print(f"{'='*60}\n")

def main():
    print("🚀 Starting UI/UX Knowledge Base Import\n")
    
    # Check if Ollama is running
    try:
        result = subprocess.run(
            ["ollama", "list"],
            capture_output=True,
            timeout=5
        )
        if result.returncode != 0:
            print("❌ Ollama is not running. Please start it with: ollama serve")
            return
    except Exception as e:
        print(f"❌ Error checking Ollama: {e}")
        return
    
    # Load chunks
    print(f"📖 Loading chunks from {CHUNKS_FILE}...")
    chunks = []
    with open(CHUNKS_FILE, 'r', encoding='utf-8') as f:
        for line in f:
            chunks.append(json.loads(line))
    
    print(f"✅ Loaded {len(chunks)} chunks\n")
    
    # Import to ZeroClaw
    import_to_zeroclaw(chunks)

if __name__ == "__main__":
    main()
