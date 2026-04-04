import os

for key, value in sorted(os.environ.items()):
    print(f"{key}={value}")
