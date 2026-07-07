import json
from pathlib import Path
from typing import Optional

from .base import BaseConverter, ConversionResult


class JupyterConverter(BaseConverter):
    """Convert Jupyter notebook (.ipynb) files to Markdown."""

    EXTENSIONS = {".ipynb"}
    MIME_TYPES = {"application/x-ipynb+json"}

    @classmethod
    def accepts(cls, mime_type: str, extension: Optional[str] = None) -> bool:
        if mime_type in cls.MIME_TYPES:
            return True
        if extension and extension.lower() in cls.EXTENSIONS:
            return True
        return False

    def convert(self, path: str, **kwargs) -> ConversionResult:
        path_obj = Path(path)
        if not path_obj.exists():
            return ConversionResult(
                text="",
                error="File not found",
                error_message=f"{path} does not exist",
            )

        try:
            raw = path_obj.read_bytes()
            try:
                content = raw.decode("utf-8")
            except UnicodeDecodeError:
                try:
                    from charset_normalizer import from_bytes
                    result = from_bytes(raw)
                    content = str(result.best())
                except ImportError:
                    content = raw.decode("latin-1")

            nb = json.loads(content)

            title = path_obj.stem
            if nb.get("metadata", {}).get("title"):
                title = nb["metadata"]["title"]

            md_parts: list[str] = []
            md_parts.append(f"# {title}\n")

            lang_info = nb.get("metadata", {}).get("language_info", {})
            kernel_info = nb.get("metadata", {}).get("kernelspec", {})
            if kernel_info:
                md_parts.append(
                    f"*Kernel: {kernel_info.get('display_name', 'Unknown')}*"
                )
                md_parts.append("")

            for cell in nb.get("cells", []):
                cell_type = cell.get("cell_type", "code")
                source = "".join(cell.get("source", []))

                if cell_type == "markdown":
                    if source.strip():
                        md_parts.append(source)
                        md_parts.append("")

                elif cell_type == "code":
                    if source.strip():
                        md_parts.append("```" + lang_info.get("name", "") + "\n" + source + "\n```")
                        md_parts.append("")

                    for output in cell.get("outputs", []):
                        otype = output.get("output_type", "")
                        if otype == "stream":
                            text = "".join(output.get("text", []))
                            if text.strip():
                                md_parts.append("```\n" + text + "\n```")
                                md_parts.append("")
                        elif otype in ("execute_result", "display_data"):
                            data = output.get("data", {})
                            pref = (
                                data.get("text/markdown")
                                or data.get("text/plain")
                                or data.get("text/html")
                            )
                            if pref:
                                text = "".join(pref) if isinstance(pref, list) else pref
                                if otype == "execute_result" and "text/markdown" not in data:
                                    md_parts.append("```\n" + text + "\n```")
                                else:
                                    md_parts.append(text)
                                md_parts.append("")
                        elif otype == "error":
                            ename = output.get("ename", "Error")
                            evalue = output.get("evalue", "")
                            md_parts.append(f"**{ename}**: {evalue}")
                            tb = "".join(output.get("traceback", []))
                            if tb:
                                md_parts.append("```\n" + tb + "\n```")
                            md_parts.append("")

            text = "\n".join(md_parts)

            return ConversionResult(
                text=text,
                title=title,
                metadata={
                    "source": str(path_obj),
                    "cells": len(nb.get("cells", [])),
                    "kernel": kernel_info.get("display_name", "") if kernel_info else "",
                    "language": lang_info.get("name", ""),
                },
            )

        except json.JSONDecodeError as e:
            return ConversionResult(
                text="",
                error="Invalid notebook JSON",
                error_message=str(e),
            )
        except Exception as e:
            return ConversionResult(
                text="",
                error="Notebook conversion failed",
                error_message=str(e),
            )
