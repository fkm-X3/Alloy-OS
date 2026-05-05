import argparse
import json
import os
import re
import subprocess
import sys
import textwrap
from pathlib import Path
from typing import Optional

GITHUB_TOKEN      = os.getenv("GITHUB_TOKEN", "")
GITHUB_REPOSITORY = os.getenv("GITHUB_REPOSITORY", "")  

MAX_CHARS         = int(os.getenv("REVIEW_MAX_TOKENS", "80000"))
ISSUE_LABEL       = os.getenv("REVIEW_LABEL", "ai-review")

DEFAULT_EXTENSIONS = {
    ".py", ".js", ".ts", ".jsx", ".tsx", ".go", ".rs", ".java",
    ".c", ".cpp", ".h", ".hpp", ".cs", ".rb", ".php", ".sh",
    ".asm", ".s", ".yaml", ".yml", ".json", ".toml", ".tf", ".sql", ".md", ".cfg",
    "Makefile", "Dockerfile", ".gitignore",
}

SKIP_DIRS = {
    ".git", ".github", "node_modules", "__pycache__", ".venv",
    "venv", "dist", "build", ".next", "target", "vendor",
    ".mypy_cache", ".pytest_cache", "coverage",
}

SYSTEM_PROMPT = textwrap.dedent("""
You are an expert security auditor and code reviewer.
Analyse the supplied codebase snapshot and return ONLY a JSON array.
Each element must be a JSON object with exactly these keys:
  "title"    : short, specific issue title (≤ 80 chars)
  "severity" : one of "critical", "high", "medium", "low", "info"
  "category" : one of "security", "bug", "performance", "maintainability", "dependency"
  "body"     : GitHub-flavoured Markdown body.
               Include: description, affected file(s)/line(s) if determinable,
               reproduction steps or proof-of-concept where relevant,
               and a concrete remediation recommendation.

Focus on real problems — do NOT invent issues.
Return [] if there is nothing to report.
Return ONLY the JSON array, no prose, no markdown fences.
""").strip()

def collect_files(root: Path, extensions: set[str]) -> list[Path]:
    files = []
    for path in sorted(root.rglob("*")):
        if any(part in SKIP_DIRS for part in path.parts):
            continue
        if not path.is_file():
            continue
        suffix = path.suffix.lower()
        if suffix in extensions or path.name in extensions:
            files.append(path)
    return files

def build_codebase_dump(root: Path, extensions: set[str]) -> str:
    files = collect_files(root, extensions)
    parts: list[str] = []
    total = 0
    skipped = 0

    for fpath in files:
        try:
            content = fpath.read_text(errors="replace")
        except Exception:
            continue
        rel = fpath.relative_to(root)
        chunk = f"### {rel}\n```{fpath.suffix.lstrip('.')}\n{content}\n```\n"
        if total + len(chunk) > MAX_CHARS:
            skipped += 1
            continue
        parts.append(chunk)
        total += len(chunk)

    header = f"# Codebase snapshot ({len(parts)} files, {total} chars)\n\n"
    if skipped:
        header += f"> {skipped} file(s) omitted due to size limit.\n\n"
    return header + "\n".join(parts)

def call_g4f(codebase: str) -> str:
    """
    Call g4f with auto provider selection.
    Falls back through a priority list if the default auto fails.
    """
    try:
        import g4f

        from g4f.client import Client
    except ImportError:
        sys.exit(
            "ERROR: g4f is not installed.\n"
            "Run:  pip install g4f\n"
            "  or: pip install -r requirements-review.txt"
        )

    client = Client()

    # Allow the user to supply a G4F API key via environment variables.
    # Common names supported: G4F_API_KEY, G4F_KEY, OPENAI_API_KEY
    api_key = os.getenv("G4F_API_KEY") or os.getenv("G4F_KEY") or os.getenv("OPENAI_API_KEY")
    extra_kwargs = {}
    if api_key:
        extra_kwargs["api_key"] = api_key
        print("Using G4F API key from environment.", flush=True)

    messages = [
        {"role": "system", "content": SYSTEM_PROMPT},
        {"role": "user",   "content": f"Review this codebase:\n\n{codebase}"},
    ]

    print("Sending codebase to g4f (auto provider) ...", flush=True)
    try:
        response = client.chat.completions.create(
            model    = "gpt-4o",

            messages = messages,
            stream   = False,
            **extra_kwargs,
        )
        return response.choices[0].message.content
    except Exception as exc:
        print(f"  Auto provider failed ({exc}), trying fallback providers …", flush=True)

        fallback_providers = ["Blackbox", "DeepInfra", "Liaobots", "Copilot"]
        for pname in fallback_providers:
            try:
                provider_cls = getattr(g4f.Provider, pname, None)
                if provider_cls is None:
                    continue
                response = client.chat.completions.create(
                    model    = "gpt-4o",
                    messages = messages,
                    provider = provider_cls,
                    stream   = False,
                    **extra_kwargs,
                )
                print(f"  Used fallback provider: {pname}", flush=True)
                return response.choices[0].message.content
            except Exception as ferr:
                print(f"  {pname}: {ferr}", flush=True)
        sys.exit("ERROR: All g4f providers failed. Cannot continue.")

def parse_issues(raw: str) -> list[dict]:

    raw = re.sub(r"^```[a-z]*\s*", "", raw.strip(), flags=re.MULTILINE)
    raw = re.sub(r"```\s*$", "", raw.strip(), flags=re.MULTILINE)
    raw = raw.strip()

    try:
        data = json.loads(raw)
    except json.JSONDecodeError:

        m = re.search(r"\[.*\]", raw, re.DOTALL)
        if m:
            try:
                data = json.loads(m.group())
            except json.JSONDecodeError:
                print("WARNING: Could not parse AI response as JSON.", file=sys.stderr)
                print("Raw response:\n", raw[:2000], file=sys.stderr)
                return []
        else:
            print("WARNING: AI returned no JSON array.", file=sys.stderr)
            return []

    if not isinstance(data, list):
        return []
    return [i for i in data if isinstance(i, dict)]

def gh_api(method: str, path: str, payload: Optional[dict] = None) -> dict:
    """Thin wrapper around the GitHub REST API."""
    import urllib.request
    import urllib.error

    url = f"https://api.github.com{path}"
    headers = {
        "Authorization": f"Bearer {GITHUB_TOKEN}",
        "Accept":        "application/vnd.github+json",
        "Content-Type":  "application/json",
        "X-GitHub-Api-Version": "2022-11-28",
    }
    body = json.dumps(payload).encode() if payload else None
    req  = urllib.request.Request(url, data=body, headers=headers, method=method)
    try:
        with urllib.request.urlopen(req) as resp:
            return json.loads(resp.read())
    except urllib.error.HTTPError as e:
        err = e.read().decode()
        print(f"GitHub API error {e.code}: {err}", file=sys.stderr)
        raise

def ensure_label(repo: str) -> None:
    """Create the review label if it doesn't exist yet."""
    try:
        gh_api("GET", f"/repos/{repo}/labels/{ISSUE_LABEL}")
    except Exception:
        try:
            gh_api("POST", f"/repos/{repo}/labels", {
                "name":  ISSUE_LABEL,
                "color": "e11d48",
                "description": "Created by ai_review.py automated scan",
            })
        except Exception:
            pass  

def get_open_issue_titles(repo: str) -> set[str]:
    """Return titles of currently open issues with our label to avoid duplicates."""
    titles: set[str] = set()
    page = 1
    while True:
        items = gh_api("GET", f"/repos/{repo}/issues?state=open&labels={ISSUE_LABEL}&per_page=100&page={page}")
        if not items:
            break
        for item in items:
            titles.add(item.get("title", "").strip())
        if len(items) < 100:
            break
        page += 1
    return titles

def create_issue(repo: str, issue: dict, commit_sha: str) -> str:
    severity = issue.get("severity", "info").upper()
    category = issue.get("category", "general")
    title    = f"[{severity}][{category}] {issue.get('title', 'Untitled issue')}"

    body = issue.get("body", "_(no description)_")
    body += f"\n\n---\n_Detected by `ai_review.py` on commit `{commit_sha[:8]}`_"

    result = gh_api("POST", f"/repos/{repo}/issues", {
        "title":  title,
        "body":   body,
        "labels": [ISSUE_LABEL],
    })
    return result.get("html_url", "")

def current_commit_sha() -> str:
    try:
        return subprocess.check_output(
            ["git", "rev-parse", "HEAD"], text=True
        ).strip()
    except Exception:
        return "unknown"

def main() -> None:
    parser = argparse.ArgumentParser(description="AI code review - GitHub Issues")
    parser.add_argument("--dry-run",     action="store_true",
                        help="Print issues but do NOT create GitHub issues")
    parser.add_argument("--max-files",   type=int, default=None,
                        help="Limit number of files sent to AI")
    parser.add_argument("--extensions",  nargs="+", default=None,
                        help="File extensions to include (e.g. .py .js)")
    parser.add_argument("--root",        default=".",
                        help="Root directory to scan (default: current dir)")
    args = parser.parse_args()

    if not args.dry_run:
        if not GITHUB_TOKEN:
            sys.exit("ERROR: GITHUB_TOKEN env var is not set.")
        if not GITHUB_REPOSITORY:
            sys.exit("ERROR: GITHUB_REPOSITORY env var is not set (e.g. 'owner/repo').")

    root       = Path(args.root).resolve()
    extensions = set(args.extensions) if args.extensions else DEFAULT_EXTENSIONS
    commit_sha = current_commit_sha()

    print(f"Scanning: {root}")
    print(f"Commit:   {commit_sha[:12]}")

    codebase = build_codebase_dump(root, extensions)
    file_count = codebase.count("### ")
    print(f"Files included: {file_count}  ({len(codebase):,} chars)")

    if args.max_files:

        files = collect_files(root, extensions)[: args.max_files]
        parts = []
        total = 0
        for fpath in files:
            try:
                content = fpath.read_text(errors="replace")
            except Exception:
                continue
            rel   = fpath.relative_to(root)
            chunk = f"### {rel}\n```{fpath.suffix.lstrip('.')}\n{content}\n```\n"
            parts.append(chunk)
            total += len(chunk)
        codebase = f"# Codebase snapshot ({len(parts)} files)\n\n" + "\n".join(parts)

    raw_response = call_g4f(codebase)
    issues       = parse_issues(raw_response)

    if not issues:
        print("No issues found.")
        return

    SEV_ORDER = {"critical": 0, "high": 1, "medium": 2, "low": 3, "info": 4}
    issues.sort(key=lambda i: SEV_ORDER.get(i.get("severity", "info"), 99))

    print(f"\n{len(issues)} issue(s) found:\n")
    for i, issue in enumerate(issues, 1):
        sev = issue.get("severity", "?").upper()
        cat = issue.get("category", "?")
        ttl = issue.get("title", "Untitled")
        print(f"  {i:2}. [{sev}][{cat}] {ttl}")

    if args.dry_run:
        print("\nDry-run mode — no GitHub issues created.")
        print("\nFull details:\n")
        for issue in issues:
            print(json.dumps(issue, indent=2))
        return

    ensure_label(GITHUB_REPOSITORY)
    existing = get_open_issue_titles(GITHUB_REPOSITORY)
    created  = 0
    skipped  = 0

    for issue in issues:
        severity = issue.get("severity", "info").upper()
        category = issue.get("category", "general")
        title    = f"[{severity}][{category}] {issue.get('title', 'Untitled issue')}"

        if title in existing:
            print(f"  Skipped (already open): {title}")
            skipped += 1
            continue

        url = create_issue(GITHUB_REPOSITORY, issue, commit_sha)
        print(f"  Created: {url}")
        created += 1

    print(f"\nDone — {created} issue(s) created, {skipped} duplicate(s) skipped.")

if __name__ == "__main__":
    main()