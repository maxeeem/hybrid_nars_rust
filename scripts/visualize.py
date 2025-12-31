import json
import numpy as np
import matplotlib.pyplot as plt
from sklearn.manifold import TSNE
import sys
import os

def visualize(filename):
    if not os.path.exists(filename):
        print(f"File {filename} not found.")
        return

    try:
        with open(filename, 'r') as f:
            data = json.load(f)
    except Exception as e:
        print(f"Error reading file: {e}")
        return

    if not data:
        print("No concepts to visualize.")
        return

    vectors = []
    labels = []
    
    print(f"Loading {len(data)} concepts...")

    for concept in data:
        # Extract term string representation
        # The term structure in JSON might be complex: {"Atom": ...} or {"Compound": ...}
        term_data = concept['term']
        if isinstance(term_data, dict):
            if 'Atom' in term_data:
                label = f"Atom({term_data['Atom']})"
            elif 'Compound' in term_data:
                label = "Compound(...)" # Simplify for now
            else:
                label = str(term_data)
        else:
            label = str(term_data)
            
        labels.append(label)
        
        # Extract vector bits
        # The Rust Hypervector struct has 'bits': [u64; 157]
        u64_bits = concept['vector']['bits']
        
        # Convert u64 array to bit array
        bit_array = []
        for u64_val in u64_bits:
            for i in range(64):
                if (u64_val >> i) & 1:
                    bit_array.append(1)
                else:
                    bit_array.append(0)
        
        vectors.append(bit_array)

    X = np.array(vectors)
    
    # t-SNE requires at least 2 samples
    if len(X) < 2:
        print("Not enough data points for t-SNE (need at least 2).")
        return

    print("Reducing dimensions with t-SNE...")
    # Perplexity must be less than number of samples
    perplexity = min(30, len(X) - 1)
    tsne = TSNE(n_components=2, random_state=42, perplexity=perplexity, init='random', learning_rate='auto')
    X_embedded = tsne.fit_transform(X)

    print("Plotting...")
    plt.figure(figsize=(12, 8))
    plt.scatter(X_embedded[:, 0], X_embedded[:, 1], alpha=0.6)

    for i, label in enumerate(labels):
        plt.annotate(label, (X_embedded[i, 0], X_embedded[i, 1]), fontsize=8, alpha=0.8)

    plt.title("Concept Memory Visualization (t-SNE)")
    plt.xlabel("Dimension 1")
    plt.ylabel("Dimension 2")
    plt.grid(True, alpha=0.3)
    
    output_file = filename.replace('.json', '.png')
    plt.savefig(output_file)
    print(f"Visualization saved to {output_file}")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python visualize.py <concepts.json>")
    else:
        visualize(sys.argv[1])
