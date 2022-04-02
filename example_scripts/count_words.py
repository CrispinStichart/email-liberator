"""
Keep track of words in all emails seen, storing the count in a file.

It expects to recieve just the body of the email.
"""

import sys
import json

if __name__ == "__main__":
    count = len(sys.stdin.readline())

    try:
        with open("/tmp/email_word_count", "r") as f:
            current = int(f.readline().strip())
            count += current
    except FileNotFoundError:
        pass

    with open("/tmp/email_word_count", "w") as f:
        f.write(str(count))
