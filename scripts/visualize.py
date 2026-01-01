import json
import sys
import numpy as np
import matplotlib.pyplot as plt
from sklearn.manifold import TSNE
from sklearn.decomposition import PCA

def load_memory(filename):
    with open(filename, 'r') as f:
        data = json.load(f)
    return data

def decode_vectors(data):
    vectors = []
    terms = []
    usages = []
    
    print("Decoding vectors...")
    for i, item in enumerate(data):
        if i % 1000 == 0:
            print(f"\rDecoded {i}/{len(data)}...", end="")
            
        terms.append(item['term'])
        usages.append(item['usage'])
        
        raw_u64 = item['vector']
        arr_u64 = np.array(raw_u64, dtype=np.uint64)
        arr_u8 = arr_u64.view(np.uint8)
        bits = np.unpackbits(arr_u8)
        
        vectors.append(bits)
    
    print("\nDone decoding.")
    return np.array(vectors), terms, usages

def visualize(filename):
    print(f"Loading {filename}...")
    data = load_memory(filename)
    
    if not data:
        print("No data found.")
        return

    print(f"Found {len(data)} concepts.")
    
    X, terms, usages = decode_vectors(data)
    
    print(f"Original Vector shape: {X.shape}")
    
    # 1. PCA Reduction (Critical for speed with high-dim data)
    # Reduce from ~10,000 dims to 50 dims
    n_pca = min(50, X.shape[0])
    print(f"Running PCA to reduce dimensions to {n_pca}...")
    pca = PCA(n_components=n_pca)
    X_pca = pca.fit_transform(X.astype(np.float32))
    print(f"PCA shape: {X_pca.shape}")

    # 2. t-SNE
    print("Running t-SNE...")
    perplexity = min(30, len(data) - 1) if len(data) > 1 else 1
    tsne = TSNE(n_components=2, random_state=42, perplexity=perplexity, init='pca', learning_rate='auto', verbose=1)
    X_embedded = tsne.fit_transform(X_pca)
    
    print("Plotting...")
    plt.figure(figsize=(16, 12))
    
    # Normalize usages for color mapping
    usages = np.array(usages)
    if len(usages) > 0 and usages.max() > usages.min():
        norm_usages = (usages - usages.min()) / (usages.max() - usages.min())
    else:
        norm_usages = np.zeros_like(usages, dtype=float)
        
    # Plot all points with low alpha
    scatter = plt.scatter(X_embedded[:, 0], X_embedded[:, 1], c=norm_usages, cmap='coolwarm', s=20, alpha=0.4)
    plt.colorbar(scatter, label='Usage / Activity')
    
    # Annotate only interesting terms (high usage or specific keywords)
    # For the demo, we want to see the animals and vehicles
    target_keywords = ["cat", "dog", "wolf", "animal", "car", "truck", "bicycle", "vehicle"]
    
    annotated_count = 0
    for i, term in enumerate(terms):
        # Clean up term string to check for keywords
        # Term format might be 'Atom("cat")' or just 'cat' depending on export
        term_lower = term.lower()
        
        is_target = any(k in term_lower for k in target_keywords)
        is_high_usage = norm_usages[i] > 0.8
        
        if is_target or is_high_usage:
            plt.annotate(term, (X_embedded[i, 0], X_embedded[i, 1]), 
                         xytext=(5, 5), textcoords='offset points',
                         fontsize=12 if is_target else 8,
                         fontweight='bold' if is_target else 'normal',
                         bbox=dict(boxstyle="round,pad=0.3", fc="white", ec="black", alpha=0.8))
            annotated_count += 1
            
    print(f"Annotated {annotated_count} concepts.")
        
    plt.title(f"Memory Visualization (PCA + t-SNE) - {len(data)} Concepts")
    plt.xlabel("Dimension 1")
    plt.ylabel("Dimension 2")
    plt.grid(True, alpha=0.3)
    plt.tight_layout()
    
    output_file = filename.replace('.json', '.png')
    plt.savefig(output_file)
    print(f"Plot saved to {output_file}")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python visualize.py <memory.json>")
        sys.exit(1)
        
    visualize(sys.argv[1])
