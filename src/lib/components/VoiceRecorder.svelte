<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  let {
    onTranscript = (_text: string) => {},
  }: {
    onTranscript?: (text: string) => void;
  } = $props();

  let isRecording = $state(false);
  let isTranscribing = $state(false);
  let transcribedText = $state<string | null>(null);
  let error = $state<string | null>(null);
  let voiceAvailable = $state(true);
  let showResult = $state(false);

  // Check if voice feature is available on mount
  $effect(() => {
    invoke<boolean>('voice_is_recording')
      .then(() => { voiceAvailable = true; })
      .catch(() => { voiceAvailable = false; });
  });

  async function toggleRecording() {
    if (isRecording) {
      await stopRecording();
    } else {
      await startRecording();
    }
  }

  async function startRecording() {
    error = null;
    transcribedText = null;
    showResult = false;

    try {
      await invoke('voice_start_recording');
      isRecording = true;
    } catch (err) {
      error = String(err);
      voiceAvailable = false;
    }
  }

  async function stopRecording() {
    isTranscribing = true;

    try {
      const result = await invoke<{ text: string; confidence: number; language: string }>(
        'voice_stop_recording',
      );
      isRecording = false;
      isTranscribing = false;

      if (result.text) {
        transcribedText = result.text;
        showResult = true;
      } else {
        error = 'No speech detected';
      }
    } catch (err) {
      isRecording = false;
      isTranscribing = false;
      error = String(err);
    }
  }

  function handleInsert() {
    if (transcribedText) {
      onTranscript(transcribedText);
      showResult = false;
      transcribedText = null;
    }
  }

  function dismissResult() {
    showResult = false;
    transcribedText = null;
  }
</script>

{#if voiceAvailable}
  <div class="voice-recorder" class:recording={isRecording}>
    <button
      class="mic-btn"
      class:recording={isRecording}
      class:transcribing={isTranscribing}
      onclick={toggleRecording}
      disabled={isTranscribing}
      title={isRecording ? 'Stop recording' : 'Start voice dictation'}
      aria-label={isRecording ? 'Stop recording' : 'Start voice dictation'}
    >
      {#if isTranscribing}
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" class="spinner">
          <circle cx="12" cy="12" r="10" opacity="0.3" />
          <path d="M12 2a10 10 0 0 1 10 10" />
        </svg>
      {:else}
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
          <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
          <line x1="12" y1="19" x2="12" y2="22" />
        </svg>
      {/if}
    </button>

    {#if isRecording}
      <div class="recording-indicator">
        <span class="recording-pulse" />
        <span class="recording-label">Recording…</span>
      </div>
    {/if}

    {#if error}
      <div class="voice-error text-sm">{error}</div>
    {/if}

    {#if showResult && transcribedText}
      <div class="transcription-result">
        <div class="transcription-text">{transcribedText}</div>
        <div class="transcription-actions">
          <button class="btn btn-primary btn-sm" onclick={handleInsert}>
            <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
              <path d="M8 3v10M3 8h10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            </svg>
            Insert at cursor
          </button>
          <button class="btn btn-ghost btn-sm" onclick={dismissResult}>Dismiss</button>
        </div>
      </div>
    {/if}
  </div>
{:else}
  <!-- Voice feature not available – render nothing or a disabled icon -->
{/if}

<style>
  .voice-recorder {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    position: relative;
  }
  .mic-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    color: var(--color-text-muted);
    transition: all var(--transition-fast);
    position: relative;
  }
  .mic-btn:hover {
    background: color-mix(in srgb, var(--color-accent) 10%, transparent);
    color: var(--color-text);
  }
  .mic-btn.recording {
    color: var(--color-error);
    background: color-mix(in srgb, var(--color-error) 12%, transparent);
    animation: mic-pulse 1.5s ease-in-out infinite;
  }
  .mic-btn.transcribing {
    color: var(--color-accent);
  }
  .mic-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  .recording-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .recording-pulse {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--color-error);
    animation: pulse-dot 1.5s ease-in-out infinite;
  }
  .recording-label {
    font-size: 11px;
    font-weight: 500;
    color: var(--color-error);
  }
  .voice-error {
    color: var(--color-error);
    padding: 2px 6px;
    background: var(--color-error-bg);
    border-radius: var(--radius-sm);
  }
  .transcription-result {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin-top: 4px;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
    padding: 8px 10px;
    z-index: var(--z-tooltip, 60);
    min-width: 240px;
  }
  .transcription-text {
    font-size: 13px;
    line-height: 1.5;
    margin-bottom: 8px;
    color: var(--color-text);
  }
  .transcription-actions {
    display: flex;
    gap: 6px;
  }
  .btn-sm {
    font-size: 11px;
    padding: 4px 10px;
  }

  @keyframes mic-pulse {
    0%, 100% { transform: scale(1); }
    50% { transform: scale(1.1); }
  }
  @keyframes pulse-dot {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.5; transform: scale(1.3); }
  }
  .spinner {
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
