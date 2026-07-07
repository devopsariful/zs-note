# 001-voice-spec.md
## ZarishNote Voice Engine Specification
### Whisper-backed transcription with Markdown export

**Document type:** Specification — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## 1. Overview

ZarishNote integrates **Whisper.cpp** (a portable, CPU-only C++ implementation of OpenAI Whisper) for local on-device speech transcription. No API key or internet connection required.

The voice engine:
- Records from microphone directly in the app
- Transcribes in near real-time
- Outputs clean Markdown
- Supports 99 languages (Whisper's language support)
- Optionally exports as SRT for subtitles

---

## 2. Architecture

```
[Microphone] → [Tauri Audio Plugin] → [whisper.cpp via Rust FFI]
                                             ↓
                                    [Transcription segments]
                                             ↓
                              [Markdown formatter (Rust)]
                                             ↓
                                    [Insert into editor]
```

Whisper.cpp runs entirely in the Rust backend. No Python. No external process.

---

## 3. Model Options

| Model | Size | RAM needed | Speed | Accuracy |
|---|---|---|---|---|
| `whisper-tiny` | 75MB | ~250MB | Fastest | Basic |
| `whisper-base` | 142MB | ~450MB | Fast | Good (default) |
| `whisper-small` | 466MB | ~1GB | Medium | Better |
| `whisper-medium` | 1.5GB | ~3GB | Slow | Best for V1 |

Given target hardware (i3 / 8GB RAM / Ubuntu), **`whisper-base` is the default**. Users can switch in Settings → Voice.

Model files downloaded on first use, cached in app data folder.

---

## 4. Transcription Modes

### 4.1 Live Dictation

- Press `Mic` button in editor toolbar (or `Cmd/Ctrl + Shift + V`)
- Recording indicator appears in status bar
- Text appears in document as you speak
- Press again to stop
- Auto-punctuation via Whisper's punctuation output
- Transcription runs in segments (every ~5 seconds)

### 4.2 File Import

- Import an audio/video file (MP3, WAV, M4A, MP4, WebM)
- Transcription runs in background
- Output saved as Markdown file in `transcripts/` folder
- Progress shown in status bar

### 4.3 Recording + Transcript

- Record a session (e.g., a meeting) inside ZarishNote
- Audio saved as WAV to `recordings/` folder
- Transcribed automatically after stop

---

## 5. Markdown Output Format

### 5.1 Simple (default)

```markdown
Meeting Notes — 2026-06-08

First topic discussed was the deployment timeline.
The team agreed to target end of Q3 for the initial release.

Action items include reviewing the architecture document
and scheduling a follow-up with the technical lead.
```

### 5.2 Timestamped

```markdown
## Transcription — 2026-06-08 14:30

[00:00:05] First topic discussed was the deployment timeline.
[00:00:18] The team agreed to target end of Q3 for the initial release.
[00:00:32] Action items include reviewing the architecture document.
```

### 5.3 SRT Export

```
1
00:00:05,000 --> 00:00:10,000
First topic discussed was the deployment timeline.

2
00:00:10,000 --> 00:00:18,000
The team agreed to target end of Q3 for the initial release.
```

---

## 6. Speaker Labels (Phase 2)

Phase 2 will add speaker diarization:
- Uses pyannote.audio (MIT) for speaker separation
- Each speaker gets a label: `[Speaker 1]`, `[Speaker 2]`, etc.
- Users can rename speakers after transcription
- Output:

```markdown
**[Ariful]:** First topic discussed was the deployment timeline.

**[Colleague]:** The team agreed to target end of Q3.
```

---

## 7. Language Support

Whisper supports 99 languages. Set in `.znrc`:
```yaml
voice:
  language: "en"    # ISO 639-1 code
```

Or auto-detect (Whisper's built-in language detection):
```yaml
voice:
  language: "auto"
```

Bangla (`bn`), Arabic (`ar`), and French (`fr`) are tested and supported.

---

## 8. Settings

| Setting | Type | Default | Description |
|---|---|---|---|
| `voice.model` | enum | `whisper-base` | Model size |
| `voice.language` | string | `en` | Language or `auto` |
| `voice.output_format` | enum | `markdown` | `markdown`, `plain`, `srt` |
| `voice.timestamp_granularity` | enum | `sentence` | `word`, `sentence`, `paragraph` |
| `voice.auto_punctuate` | bool | true | Use Whisper's punctuation |
| `voice.output_folder` | string | `transcripts` | Relative to vault root |
| `voice.speaker_labels` | bool | false | Phase 2 only |

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*