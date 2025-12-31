import random

# Generate a fake glove.txt with 50 dimensions
# Format: word f1 f2 ... f50
words = ["tiger", "cat", "dog", "wolf", "car", "truck"]
vectors = {}

# Make tiger/cat similar
vectors["tiger"] = [1.0] * 25 + [0.0] * 25
vectors["cat"]   = [0.9] * 25 + [0.1] * 25

# Make dog/wolf similar but different from cat
vectors["dog"]   = [0.0] * 25 + [1.0] * 25
vectors["wolf"]  = [0.1] * 25 + [0.9] * 25

# Make car/truck orthogonal to animals
vectors["car"]   = [0.5] * 50
vectors["truck"] = [0.5] * 50

with open("assets/glove.txt", "w") as f:
    for word, vec in vectors.items():
        line = f"{word} " + " ".join(f"{x:.4f}" for x in vec)
        f.write(line + "\n")

print("Created assets/glove.txt with 6 concepts (50 dim).")
