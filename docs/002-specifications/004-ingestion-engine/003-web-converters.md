# 003-web-converters.md
## ZarishNote Web Converters Specification
### YouTube, Wikipedia, RSS, and SERP conversion

**Document type:** Specification — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

1. [Web Converter Model](#1-web-converter-model)
2. [YouTube Converter](#2-youtube-converter)
3. [Wikipedia Converter](#3-wikipedia-converter)
4. [RSS/Atom Feed Converter](#4-rssatom-feed-converter)
5. [SERP / Bing Search Converter](#5-serp--bing-search-converter)
6. [Generic HTML Converter](#6-generic-html-converter)
7. [Configuration](#7-configuration)

---

## 1. Web Converter Model

Web converters extend the Ingestion Engine's converter registry. They accept URLs instead of file paths. Detection is based on URL pattern matching (regex) rather than MIME type.

All web converters implement:

```python
class WebConverter(BaseConverter):
    priority: float = 5.0

    def accepts(self, source: str, mime_type: str = None) -> bool:
        """Match URL pattern via regex."""

    async def convert(self, source: str, **kwargs) -> ConversionResult:
        """Fetch URL, extract content, return Markdown."""
```

Web converters are async (HTTP fetch) while file converters are sync.

---

## 2. YouTube Converter

| Field | Detail |
|---|---|
| URL patterns | `youtube.com/watch?v=*`, `youtu.be/*`, `youtube.com/embed/*` |
| Method | `youtube-transcript-api` for transcript + `yt-dlp` (optional) for metadata |
| Output | Title, channel name, description (first 200 chars), transcript as Markdown |

### 2.1 Output Format

```markdown
# Video: Understanding the Rohingya Crisis

**Channel:** UNHCR
**URL:** https://youtube.com/watch?v=abc123
**Duration:** 12:34

## Description

An overview of the humanitarian response in Cox's Bazar.

## Transcript

[00:00] The Rohingya refugee crisis began in August 2017...
[00:15] Today, over one million refugees live in the world's largest camp...

*(Auto-transcribed. Timestamps are approximate.)*
```

### 2.2 Modes

| Mode | Behavior |
|---|---|
| `metadata-only` | Title, description, thumbnail URL. No transcript fetch. |
| `transcript` | Full transcript with timestamps (default) |
| `summarized` | Transcript passed through AI for 3-bullet summary + key quotes |

### 2.3 Dependencies

- `youtube-transcript-api` (Python, optional)
- Falls back to metadata-only if library not installed
- No API key required

---

## 3. Wikipedia Converter

| Field | Detail |
|---|---|
| URL patterns | `*.wikipedia.org/wiki/*`, `*.wikipedia.org/w/index.php?title=*` |
| Method | Wikipedia REST API (`/api/rest_v1/page/summary/{title}`) |
| Output | Title, summary, structured sections, infobox as table |

### 3.1 Output Format

```markdown
# Rohingya people

*From Wikipedia, the free encyclopedia*

## Summary

The Rohingya people are a stateless Indo-Aryan ethnic group...

## Contents

- [History](#history)
- [Demographics](#demographics)

## History

### Origins

The Rohingya have inhabited northern Rakhine State...
```

### 3.2 Features

- Section extraction: each Wikipedia section becomes a Markdown heading
- Infobox: converted to GFM Markdown table
- References: preserved as `[1]` footnotes
- Images: infobox image URL included (not downloaded)
- Language: respects Wikipedia language subdomain (`bn.wikipedia.org`, `ar.wikipedia.org`)

### 3.3 API Rate Limiting

Wikipedia API is free but rate-limited. ZarishNote respects:
- Max 1 request per second (per originating IP)
- User-Agent set to `ZarishNote/1.0 (zarishsphere-foundation)` per Wikimedia policy

---

## 4. RSS/Atom Feed Converter

| Field | Detail |
|---|---|
| URL patterns | Any URL returning RSS or Atom XML |
| Detection | MIME type (`application/rss+xml`, `application/atom+xml`) or content sniffing |
| Libraries | `feedparser` (Python) |
| Output | Feed title + each entry as a Markdown section |

### 4.1 Output Format

```markdown
# UNHCR News Feed

*Source: https://www.unhcr.org/news/rss*

## Rohingya Repatriation Delayed Again — 2026-06-01

More than one million Rohingya refugees in Bangladesh...

[Read more](https://www.unhcr.org/news/rohingya-repatriation-2026)

## Monsoon Season Threatens Camps — 2026-05-28

[Read more](https://www.unhcr.org/news/monsoon-2026)
```

### 4.2 Features

- Publ date used as heading subtext
- Full content or summary (configurable)
- Enclosures (podcasts, images) noted but not downloaded
- Feed metadata: title, description, link, last updated

---

## 5. SERP / Bing Search Converter

| Field | Detail |
|---|---|
| URL patterns | `bing.com/search?q=*` |
| Method | HTTP GET + HTML parsing (BeautifulSoup) |
| Output | Search results as Markdown list |

### 5.1 Output Format

```markdown
# Bing Search: Rohingya health guidelines

*Source: bing.com/search?q=Rohingya+health+guidelines*

1. [WHO Rohingya Crisis Health Response](https://www.who.int/emergencies/situations/rohingya-refugee-crisis) - WHO's health response plan for Rohingya refugees
2. [UNHCR Health Guidelines](https://www.unhcr.org/health) - UNHCR's health protocols for refugee settings
```

### 5.2 Limitations

- HTML scraping: fragile, may break if Bing changes layout
- No guaranteed order or completeness
- Respects `robots.txt` (ZarishNote's HTTP client checks)
- Rate-limited: max 1 request per 3 seconds

### 5.3 Alternative: MCP Search

For reliable search, use the Brave Search MCP server instead of the built-in SERP converter.

---

## 6. Generic HTML Converter

| Field | Detail |
|---|---|
| URL patterns | Any HTTP/HTTPS URL not matched by other converters |
| Method | `requests` + `markdownify` with script/style stripping |
| Output | Clean Markdown with title extracted from `<title>` tag |

### 6.1 Processing Rules

1. Fetch HTML with configurable timeout (default 10s)
2. Strip `<script>`, `<style>`, `<nav>`, `<footer>`, `<noscript>` blocks
3. Extract `<title>` as document title
4. Extract `<meta name="description">` content as description
5. Convert `<body>` → Markdown via `markdownify`
6. Base64 data URIs in images truncated by default
7. Links preserved as `[text](url)`

### 6.2 Security

- All HTML parsing uses `defusedxml` / `lxml` with safe defaults
- No JavaScript execution
- No cookie storage
- No automatic redirect following beyond standard HTTP redirects

---

## 7. Configuration

Web converters configured in `.znrc` or via Settings → Ingestion:

```yaml
ingestion:
  web:
    youtube:
      enabled: true
      mode: "transcript"          # metadata-only | transcript | summarized
      summarization_provider: "ollama"  # AI provider for summarization mode

    wikipedia:
      enabled: true
      language: "en"              # default language code

    rss:
      enabled: true
      max_entries: 20

    serp:
      enabled: true

    generic:
      timeout: 10                 # seconds
      max_size: 5242880           # 5MB max HTML response
```

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
