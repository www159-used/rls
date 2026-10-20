#!/bin/sh
# Example naming script: JSON stdin/stdout, no Git mutations. Requires Python 3.
# Copy into the target worktree; configure command = ["sh", "./scripts/next-tag.sh"].
exec python3 -c '
import json
import sys

request = json.load(sys.stdin)
if request["version"] != 1:
    raise SystemExit("Unsupported request version")
# Use the fixed release target, not the worktree HEAD or local version files.
name = "release-{}-{}".format(request["branch"], request["target_sha"][:12])
json.dump({"version": 1, "action": "tag", "name": name}, sys.stdout)
sys.stdout.write("\n")
'
