#!/usr/bin/env python3
"""
This is the launcher for the script in tools/update/main.py.

Why does this script have a launcher? By putting the entire body of
code in another module that you import later, you can allow python versions
that can't even parse the new syntax to have a pretty error message saying
that the version is too low. As we are likely to get complaints about these
scripts, having high usability is good.
"""

import os
import sys

if sys.version_info < (3, 5, 0):
    print("%s requires python > 3.5" % (sys.argv[0]))
    exit(1)

if not os.path.exists(".nova-root"):
    print("This script must be run inside nova's root directory")
    print("Cannot find .nova-root")

sys.path.insert(0, "tools/update/")

import main

if __name__ == "__main__":
    main.main()
