"""Check local Markdown links, heading anchors, plan boundaries, and fences.

This audit helper uses only the Python standard library. It handles the inline
links and ATX headings used in this repository; it is not a CommonMark parser or
an external URL checker. Run from any directory. --write records checks.json.
"""

import json
from pathlib import Path
import re
import subprocess
import sys
import unicodedata
from urllib.parse import unquote, urlsplit


TASK = Path(__file__).resolve().parent
ROOT = TASK.parents[2]
PLAN_ROOT = ROOT / "docs/plans"
tracked = subprocess.check_output(
    ["git", "ls-tree", "-r", "-z", "--name-only", "HEAD"], cwd=ROOT
).decode().split("\0")
baseline = sorted(
    ROOT / name for name in tracked
    if Path(name).suffix in (".md", ".mdx")
    and not (ROOT / name).is_relative_to(TASK)
)
documents = sorted(set(baseline) | set(TASK.glob("*.md")))
issues = []
contents = {}
anchors = {}
counts = {
    group: {"local_links": 0, "heading_links": 0, "external_links_skipped": 0}
    for group in ("baseline", "task")
}


def slug(heading):
    heading = re.sub(r"<[^>]+>", "", heading).replace("`", "")
    heading = re.sub(r"\[([^]]+)\]\([^)]*\)", r"\1", heading)
    heading = "".join(
        char for char in heading.lower()
        if char in " -_" or unicodedata.category(char)[0] in "LNM"
    )
    return heading.replace(" ", "-")


def issue(path, line, kind, target):
    issues.append({"file": str(path.relative_to(ROOT)), "line": line, "kind": kind, "target": target})


for path in documents:
    visible = []
    fence = None
    heading_ids = set()
    for number, line in enumerate(path.read_text().splitlines(), 1):
        delimiter = re.match(r"^ {0,3}(`{3,}|~{3,})(.*)$", line)
        if fence:
            if delimiter and delimiter[1][0] == fence[0] and len(delimiter[1]) >= fence[1] and not delimiter[2].strip():
                fence = None
            continue
        if delimiter:
            fence = (delimiter[1][0], len(delimiter[1]), number)
            continue
        visible.append((number, line))
        heading = re.match(r"^ {0,3}#{1,6}\s+(.+?)(?:\s+#+)?\s*$", line)
        if heading:
            base = slug(heading[1])
            candidate = base
            suffix = 0
            while candidate in heading_ids:
                suffix += 1
                candidate = f"{base}-{suffix}"
            heading_ids.add(candidate)
    if fence:
        issue(path, fence[2], "unclosed_fence", fence[0] * fence[1])
    contents[path] = visible
    anchors[path] = heading_ids

for path, lines in contents.items():
    group_counts = counts["task" if path.is_relative_to(TASK) else "baseline"]
    for number, line in lines:
        for match in re.finditer(r"!?\[[^\]\n]*\]\(([^)\n]+)\)", line):
            destination = match[1].strip().strip("<>")
            parsed = urlsplit(destination)
            if parsed.scheme or parsed.netloc:
                group_counts["external_links_skipped"] += 1
                continue
            group_counts["local_links"] += 1
            target = (path.parent / unquote(parsed.path)).resolve() if parsed.path else path
            if not target.exists():
                issue(path, number, "missing_file", destination)
                continue
            if target.is_relative_to(PLAN_ROOT) and not path.is_relative_to(PLAN_ROOT):
                issue(path, number, "permanent_to_plan", destination)
            if parsed.fragment:
                group_counts["heading_links"] += 1
                if target not in anchors or unquote(parsed.fragment) not in anchors[target]:
                    issue(path, number, "missing_anchor", destination)

result = {
    "baseline_revision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT).decode().strip(),
    "baseline_documents": len(baseline),
    "task_documents": len(documents) - len(baseline),
    "documents_checked": len(documents),
    "link_counts": counts,
    "issues": issues,
    "baseline_inventory": [
        {"path": str(path.relative_to(ROOT)), "lines": len(path.read_text().splitlines())}
        for path in baseline
    ],
}
output = json.dumps(result, indent=2, ensure_ascii=False) + "\n"
if "--write" in sys.argv[1:]:
    (TASK / "checks.json").write_text(output)
print(output, end="")
sys.exit(bool(issues))
