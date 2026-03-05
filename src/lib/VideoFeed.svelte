<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { passthroughConfig } from "./stores";

  interface VideoDevice {
    deviceId: string;
    label: string;
  }

  let videoDevices = $state<VideoDevice[]>([]);
  let selectedDeviceId = $state("");
  let videoEl: HTMLVideoElement | undefined = $state();
  let stream: MediaStream | null = $state(null);
  let videoEnabled = $state(true);
  let error = $state("");

  passthroughConfig.subscribe((c) => {
    videoEnabled = c.video;
    if (!c.video && stream) {
      stopStream();
    }
  });

  onMount(async () => {
    await loadDevices();
  });

  onDestroy(() => {
    stopStream();
  });

  async function loadDevices() {
    try {
      // Request permission first
      const tempStream = await navigator.mediaDevices.getUserMedia({ video: true });
      tempStream.getTracks().forEach((t) => t.stop());

      const allDevices = await navigator.mediaDevices.enumerateDevices();
      videoDevices = allDevices
        .filter((d) => d.kind === "videoinput")
        .map((d) => ({
          deviceId: d.deviceId,
          label: d.label || `Camera ${d.deviceId.slice(0, 8)}`,
        }));

      if (videoDevices.length > 0 && !selectedDeviceId) {
        selectedDeviceId = videoDevices[0].deviceId;
      }
    } catch (e) {
      error = `Failed to enumerate devices: ${e}`;
    }
  }

  async function startStream() {
    if (!selectedDeviceId || !videoEnabled) return;
    stopStream();
    error = "";

    try {
      stream = await navigator.mediaDevices.getUserMedia({
        video: {
          deviceId: { exact: selectedDeviceId },
          width: { ideal: 1920 },
          height: { ideal: 1080 },
          frameRate: { ideal: 60 },
        },
        audio: false,
      });

      if (videoEl) {
        videoEl.srcObject = stream;
      }
    } catch (e) {
      error = `Failed to start capture: ${e}`;
    }
  }

  function stopStream() {
    if (stream) {
      stream.getTracks().forEach((t) => t.stop());
      stream = null;
    }
    if (videoEl) {
      videoEl.srcObject = null;
    }
  }

  function onDeviceChange(e: Event) {
    selectedDeviceId = (e.target as HTMLSelectElement).value;
    if (stream) {
      startStream();
    }
  }
</script>

<div class="video-feed">
  <div class="video-toolbar">
    <select value={selectedDeviceId} onchange={onDeviceChange} disabled={!videoEnabled}>
      {#each videoDevices as device}
        <option value={device.deviceId}>{device.label}</option>
      {:else}
        <option value="">No capture devices</option>
      {/each}
    </select>

    <button onclick={startStream} disabled={!videoEnabled || !selectedDeviceId}>
      Start Capture
    </button>

    <button onclick={stopStream} disabled={!stream}>
      Stop
    </button>
  </div>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  <div class="video-container">
    <!-- svelte-ignore a11y_media_has_caption -->
    <video bind:this={videoEl} autoplay playsinline></video>

    {#if !stream && !error}
      <div class="placeholder">
        <p>No video feed active</p>
        <p class="sub">Select a capture device and click Start</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .video-feed {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .video-toolbar {
    display: flex;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    align-items: center;
    background: rgba(0, 0, 0, 0.3);
  }
  .video-toolbar select {
    flex: 1;
    padding: 0.4rem;
    background: #1e1e1e;
    color: #ccc;
    border: 1px solid #444;
    border-radius: 4px;
    font-size: 0.85rem;
  }
  .video-toolbar button {
    padding: 0.4rem 0.8rem;
    background: #3b82f6;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.85rem;
  }
  .video-toolbar button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .video-container {
    flex: 1;
    position: relative;
    background: #000;
    min-height: 0;
    overflow: hidden;
  }
  video {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }
  .placeholder {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    color: #666;
  }
  .placeholder p {
    margin: 0.2rem;
  }
  .placeholder .sub {
    font-size: 0.8rem;
    opacity: 0.6;
  }
  .error {
    color: #ef4444;
    font-size: 0.85rem;
    padding: 0 1rem;
    margin: 0.3rem 0;
  }
</style>
