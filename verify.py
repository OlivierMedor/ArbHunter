import sys
import os

files = ['Cargo.toml', 'docker-compose.yml', 'Makefile', '.env.example', 'contracts/foundry.toml']

print("--- A. Newline counts ---")
for f in files:
    if os.path.exists(f):
        with open(f, 'rb') as file:
            content = file.read()
            # Count the number of '\n' characters
            count = content.count(b'\n')
            print(f"{count} {f}")
    else:
        print(f"Missing {f}")

print("\n--- B. First 20 lines numbered ---")
for f in files:
    print(f"\n{f}:")
    if os.path.exists(f):
        with open(f, 'rb') as file:
            content = file.read().decode('utf-8')
            lines = content.split('\n')
            for i, line in enumerate(lines[:20], 1):
                # Represent carriage returns visually if they exist
                visual_line = line.replace('\r', '\\r')
                print(f"    {i}\t{visual_line}")
