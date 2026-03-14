#!/usr/bin/env python3
import json
import os
import re
import subprocess
import sys
import urllib.error
import urllib.parse
import urllib.request
from collections import Counter
from pathlib import Path
from typing import Any, Dict, List, Optional

GITHUB_API = "https://api.github.com"
OUTPUT_DIR = Path('.triage-agent/dependabot')
MAX_LINE_CONTENT_LENGTH = 500
MAX_EVIDENCE_USAGES = 100
CODE_FILE_EXTENSIONS = {
    '.rs', '.py', '.js', '.jsx', '.ts', '.tsx', '.mjs', '.cjs', '.java', '.kt',
    '.go', '.rb', '.php', '.cs', '.scala', '.swift', '.c', '.cc', '.cpp', '.h',
    '.hpp', '.toml', '.json', '.yaml', '.yml', '.xml', '.gradle', '.kts', '.lock',
    '.md'
}


def package_info(alert: Dict[str, Any]) -> Dict[str, Any]:
    return ((alert.get('dependency') or {}).get('package') or {})


def run_git(args: List[str], check: bool = True) -> str:
    result = subprocess.run(['git', *args], text=True, capture_output=True)
    if check and result.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)} failed: {result.stderr.strip()}")
    return result.stdout


def parse_link_header(link_header: str) -> Dict[str, str]:
    links: Dict[str, str] = {}
    for part in link_header.split(','):
        section = part.strip().split(';')
        if len(section) < 2:
            continue
        url = section[0].strip().strip('<>')
        rel_part = next((x.strip() for x in section[1:] if 'rel=' in x), None)
        if not rel_part:
            continue
        rel = rel_part.split('=', 1)[1].strip('"')
        links[rel] = url
    return links


def github_get(url: str, token: str) -> tuple[Any, Dict[str, str]]:
    req = urllib.request.Request(url)
    req.add_header('Accept', 'application/vnd.github+json')
    req.add_header('Authorization', f'Bearer {token}')
    req.add_header('X-GitHub-Api-Version', '2022-11-28')

    with urllib.request.urlopen(req) as response:
        body = response.read().decode('utf-8')
        headers = {k: v for (k, v) in response.getheaders()}
        return json.loads(body), headers


def fetch_open_dependabot_alerts(repo: str, token: str) -> List[Dict[str, Any]]:
    encoded_repo = urllib.parse.quote(repo, safe='/')
    next_url = f"{GITHUB_API}/repos/{encoded_repo}/dependabot/alerts?state=open&per_page=100"
    findings: List[Dict[str, Any]] = []

    while next_url:
        page, headers = github_get(next_url, token)
        if not isinstance(page, list):
            raise RuntimeError('Unexpected Dependabot alerts response format')
        findings.extend(page)

        link_header = headers.get('Link', '')
        links = parse_link_header(link_header) if link_header else {}
        next_url = links.get('next')

    return findings


def normalize_package_terms(package_name: str) -> List[str]:
    parts = [package_name]
    if '-' in package_name:
        parts.append(package_name.replace('-', '_'))
    if '_' in package_name:
        parts.append(package_name.replace('_', '-'))
    return sorted({p for p in parts if p})


def tracked_files() -> List[str]:
    output = run_git(['ls-files'])
    files = []
    for line in output.splitlines():
        path = Path(line)
        if path.parts and path.parts[0] == '.triage-agent':
            continue
        if path.suffix.lower() in CODE_FILE_EXTENSIONS or path.name in {'Cargo.toml', 'package.json', 'package-lock.json', 'pom.xml', 'go.mod', 'requirements.txt'}:
            files.append(line)
    return files


def find_package_usage_lines(package_name: str, manifest_path: Optional[str]) -> List[Dict[str, Any]]:
    terms = normalize_package_terms(package_name)
    tracked = tracked_files()
    usage: List[Dict[str, Any]] = []

    for rel_path in tracked:
        try:
            text = Path(rel_path).read_text(encoding='utf-8')
        except (UnicodeDecodeError, FileNotFoundError, PermissionError):
            continue

        lines = text.splitlines()
        for idx, line in enumerate(lines, start=1):
            if any(term in line for term in terms):
                usage.append({'file': rel_path, 'line': idx, 'content': line.strip()[:MAX_LINE_CONTENT_LENGTH]})

    if manifest_path and not any(u['file'] == manifest_path for u in usage):
        path = Path(manifest_path)
        if path.exists() and path.is_file():
            try:
                lines = path.read_text(encoding='utf-8').splitlines()
                for idx, line in enumerate(lines, start=1):
                    if any(term in line for term in terms):
                        usage.append({'file': manifest_path, 'line': idx, 'content': line.strip()[:MAX_LINE_CONTENT_LENGTH]})
            except UnicodeDecodeError:
                pass

    return usage


def blame_author_for_line(file_path: str, line_no: int) -> Optional[Dict[str, str]]:
    try:
        output = run_git(['blame', '-e', '--line-porcelain', f'-L{line_no},{line_no}', '--', file_path], check=False)
    except RuntimeError:
        return None

    if not output.strip():
        return None

    author = None
    email = None
    for line in output.splitlines():
        if line.startswith('author '):
            author = line.removeprefix('author ').strip()
        elif line.startswith('author-mail '):
            email = line.removeprefix('author-mail ').strip().strip('<>')
        if author and email:
            break

    if author or email:
        return {'name': author or 'unknown', 'email': email or 'unknown'}
    return None


def find_best_owner(usages: List[Dict[str, Any]]) -> Dict[str, str]:
    candidates: Counter[tuple[str, str]] = Counter()

    for usage in usages:
        file_path = usage['file']
        line_no = usage['line']
        author = blame_author_for_line(file_path, line_no)
        if author:
            candidates[(author['name'], author['email'])] += 1

    if not candidates:
        return {'name': 'unknown', 'email': 'unknown'}

    (name, email), _ = candidates.most_common(1)[0]
    return {'name': name, 'email': email}


def safe_alert_filename(alert: Dict[str, Any]) -> str:
    number = alert.get('number', 'unknown')
    dep_name = package_info(alert).get('name') or 'dependency'
    slug = re.sub(r'[^a-zA-Z0-9._-]+', '-', dep_name).strip('-').lower() or 'dependency'
    return f"alert-{number}-{slug}.json"


def build_record(alert: Dict[str, Any]) -> Dict[str, Any]:
    dependency_package = package_info(alert)
    package_name = dependency_package.get('name') or 'unknown'
    manifest_path = alert.get('manifest_path')
    usages = find_package_usage_lines(package_name, manifest_path)
    owner = find_best_owner(usages)

    advisory = alert.get('security_advisory') or {}

    return {
        'alert_number': alert.get('number'),
        'state': alert.get('state'),
        'severity': ((alert.get('security_vulnerability') or {}).get('severity')),
        'dependency': {
            'name': package_name,
            'ecosystem': dependency_package.get('ecosystem'),
            'manifest_path': manifest_path,
            'scope': ((alert.get('dependency') or {}).get('scope')),
        },
        'advisory': {
            'ghsa_id': advisory.get('ghsa_id'),
            'cve_id': advisory.get('cve_id'),
            'summary': advisory.get('summary'),
            'url': advisory.get('html_url'),
        },
        'recommended_user': owner,
        'evidence': {
            'matching_usage_count': len(usages),
            'matching_usages': usages[:MAX_EVIDENCE_USAGES],
            'method': 'package-name usage search + git blame on matching lines',
        },
        'analyzed_at': os.environ.get('TRIAGE_ANALYZED_AT'),
    }


def write_records(alerts: List[Dict[str, Any]]) -> None:
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    written = set()
    for alert in alerts:
        record = build_record(alert)
        filename = safe_alert_filename(alert)
        written.add(filename)
        out_path = OUTPUT_DIR / filename
        out_path.write_text(json.dumps(record, indent=2, sort_keys=True) + '\n', encoding='utf-8')

    for existing in OUTPUT_DIR.glob('*.json'):
        if existing.name not in written:
            existing.unlink()


def main() -> int:
    repo = os.environ.get('GITHUB_REPOSITORY')
    token = os.environ.get('GITHUB_TOKEN')
    alerts_file = os.environ.get('DEPENDABOT_ALERTS_FILE')

    if not repo:
        print('GITHUB_REPOSITORY is required', file=sys.stderr)
        return 2

    if not token and not alerts_file:
        print('GITHUB_TOKEN is required', file=sys.stderr)
        return 2

    if alerts_file:
        alerts = json.loads(Path(alerts_file).read_text(encoding='utf-8'))
    else:
        try:
            alerts = fetch_open_dependabot_alerts(repo, token)
        except urllib.error.HTTPError as exc:
            print(f'Failed to fetch Dependabot alerts: HTTP {exc.code}', file=sys.stderr)
            return 1
        except (RuntimeError, urllib.error.URLError, json.JSONDecodeError, ValueError) as exc:
            print(f'Failed to fetch Dependabot alerts: {exc}', file=sys.stderr)
            return 1

    write_records(alerts)
    print(f'Analyzed {len(alerts)} open Dependabot alerts')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
