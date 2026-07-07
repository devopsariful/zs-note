#!/usr/bin/env python3
"""CLI entry point for the ZarishNote Ingestion Engine."""

from __future__ import annotations

import argparse
import json
import os
import sys
import tempfile
from io import BytesIO
from pathlib import Path
from typing import Optional, Sequence

from .markitdown import ZarishNoteIngester, ConversionResult
from .exceptions import MissingDependencyError
from .mime_detect import MimeDetector


# ---------------------------------------------------------------------------
# Argument parsing
# ---------------------------------------------------------------------------

def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="zarishnote-ingest",
        description="ZarishNote Ingestion Engine — convert documents to Markdown",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=(
            "Examples:\n"
            "  zarishnote-ingest document.pdf\n"
            "  zarishnote-ingest https://example.com/article\n"
            "  zarishnote-ingest file.docx --output output.md\n"
            "  zarishnote-ingest file.html --mime text/html\n"
            "  cat file.html | zarishnote-ingest --ext html\n"
            "  zarishnote-ingest --list-converters\n"
        ),
    )

    parser.add_argument(
        "input",
        nargs="?",
        help="Input file path or URL (reads from stdin if omitted)",
    )
    parser.add_argument(
        "-o",
        "--output",
        help="Write output to FILE instead of stdout",
    )
    parser.add_argument(
        "--mime",
        help="Explicit MIME type hint (e.g. text/html)",
    )
    parser.add_argument(
        "--charset",
        help="Explicit character encoding (e.g. utf-8, latin-1)",
    )
    parser.add_argument(
        "--ext",
        help="Explicit file extension hint (e.g. .html, .csv)",
    )
    parser.add_argument(
        "--list-converters",
        action="store_true",
        help="List all available converters and exit",
    )
    parser.add_argument(
        "--no-plugins",
        action="store_true",
        help="Disable markitdown plugin discovery",
    )
    parser.add_argument(
        "--vision-model",
        help="Vision model name for image / OCR conversion",
    )
    parser.add_argument(
        "--output-json",
        action="store_true",
        help="Output result as JSON (includes metadata)",
    )

    return parser


def parse_args(args: Optional[Sequence[str]] = None) -> argparse.Namespace:
    return _build_parser().parse_args(args)


# ---------------------------------------------------------------------------
# Core helpers
# ---------------------------------------------------------------------------

def is_url(s: str) -> bool:
    return s.startswith(("http://", "https://"))


def _stdin_result(args: argparse.Namespace) -> ConversionResult:
    """Read stdin and convert."""
    if sys.stdin.isatty():
        return ConversionResult(
            text="",
            error="No input",
            error_message="Provide a file path, URL, or pipe data to stdin",
        )

    raw = sys.stdin.buffer.read()
    ingester = ZarishNoteIngester(
        plugins=not args.no_plugins,
        vision_model=args.vision_model,
    )

    # When an extension is provided, write to a temp file so the full
    # converter pipeline (magic-byte detection + custom converters) runs.
    if args.ext and not args.mime:
        suffix = args.ext if args.ext.startswith(".") else f".{args.ext}"
        fd, tmp = tempfile.mkstemp(suffix=suffix)
        try:
            os.write(fd, raw)
        finally:
            os.close(fd)
        try:
            return ingester.convert_local(tmp, mime_hint=args.mime, charset=args.charset)
        finally:
            os.unlink(tmp)

    mime_type: str = args.mime or "text/plain"
    if args.ext and not args.mime:
        mime_type = MimeDetector().detect(f"file{args.ext}")

    return ingester.convert_stream(BytesIO(raw), mime_type)


def _output_result(
    result: ConversionResult,
    output_path: Optional[str],
    as_json: bool,
) -> None:
    """Print or write the conversion result."""
    if as_json:
        payload = json.dumps(
            {
                "text": result.text,
                "title": result.title,
                "metadata": result.metadata,
                "source_url": result.source_url,
                "error": result.error,
                "error_message": result.error_message,
            },
            indent=2,
            default=str,
            ensure_ascii=False,
        )
    else:
        payload = result.text

    if output_path:
        Path(output_path).write_text(payload, encoding="utf-8")
        status = "error" if result.error else "success"
        print(f"Written to {output_path} ({status})", file=sys.stderr)
    else:
        sys.stdout.write(payload)
        if not payload.endswith("\n"):
            sys.stdout.write("\n")


def _list_converters(as_json: bool) -> None:
    """List all registered converters."""
    ingester = ZarishNoteIngester(plugins=False)
    converters = ingester.list_converters()

    if as_json:
        print(json.dumps(converters, indent=2, default=str))
        return

    header = f"{'Converter':<25} {'MIME Types':<40} {'Extensions':<20} URLs"
    print(header)
    print("-" * len(header))
    for c in converters:
        mt = ", ".join(c["mime_types"][:3])
        ext = ", ".join(c["extensions"])
        urls = "Yes" if c["supports_urls"] else "No"
        print(f"{c['name']:<25} {mt:<40} {ext:<20} {urls}")


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def main(argv: Optional[Sequence[str]] = None) -> int:
    """CLI entry point (invoked by the ``zarishnote-ingest`` console script)."""
    args = parse_args(argv)

    # --list-converters
    if args.list_converters:
        _list_converters(args.output_json)
        return 0

    # stdin mode
    if not args.input:
        result = _stdin_result(args)
        _output_result(result, args.output, args.output_json)
        return 1 if result.error else 0

    ingester = ZarishNoteIngester(
        plugins=not args.no_plugins,
        vision_model=args.vision_model,
    )

    try:
        if is_url(args.input):
            result = ingester.convert_uri(args.input, args.vision_model)
        else:
            result = ingester.convert_local(
                args.input,
                mime_hint=args.mime,
                charset=args.charset,
            )
    except MissingDependencyError as exc:
        print(f"Error: {exc}", file=sys.stderr)
        print(
            "Install the missing optional dependency, e.g.:\n"
            "    pip install 'zarishnote-ingest[all]'",
            file=sys.stderr,
        )
        return 1
    except Exception as exc:
        print(f"Unexpected error: {exc}", file=sys.stderr)
        return 1

    _output_result(result, args.output, args.output_json)
    return 1 if result.error else 0


if __name__ == "__main__":
    sys.exit(main())
