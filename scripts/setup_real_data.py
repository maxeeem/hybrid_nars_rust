import os
import urllib.request
import zipfile
import shutil

GLOVE_URL = "http://nlp.stanford.edu/data/glove.6B.zip"
ZIP_FILE = "glove.6B.zip"
TARGET_FILE = "glove.6B.50d.txt"
DEST_DIR = "assets"
DEST_FILE = os.path.join(DEST_DIR, "glove.txt")

def setup_real_data():
    if not os.path.exists(DEST_DIR):
        os.makedirs(DEST_DIR)

    if os.path.exists(DEST_FILE):
        print(f"GloVe embeddings already exist at {DEST_FILE}")
        return

    if not os.path.exists(ZIP_FILE):
        print(f"Downloading GloVe embeddings from {GLOVE_URL}...")
        try:
            urllib.request.urlretrieve(GLOVE_URL, ZIP_FILE)
            print("Download complete.")
        except Exception as e:
            print(f"Error downloading file: {e}")
            return

    print(f"Extracting {TARGET_FILE}...")
    try:
        with zipfile.ZipFile(ZIP_FILE, 'r') as zip_ref:
            zip_ref.extract(TARGET_FILE)
        
        print(f"Moving {TARGET_FILE} to {DEST_FILE}...")
        shutil.move(TARGET_FILE, DEST_FILE)
        
        print("Cleaning up...")
        os.remove(ZIP_FILE)
        # Clean up other extracted files if any (though we only extracted one)
        
        print("Setup complete.")
    except Exception as e:
        print(f"Error extracting/moving file: {e}")

if __name__ == "__main__":
    setup_real_data()
