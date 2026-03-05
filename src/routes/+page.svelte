<script lang="ts">
  import ConnectionStatus from "$lib/ConnectionStatus.svelte";
  import DeviceScanner from "$lib/DeviceScanner.svelte";
  import PassthroughControls from "$lib/PassthroughControls.svelte";
  import LockModeIndicator from "$lib/LockModeIndicator.svelte";
  import VideoFeed from "$lib/VideoFeed.svelte";
</script>

<main class="app">
  <header class="top-bar">
    <div class="brand">EmulStick</div>
    <ConnectionStatus />
    <LockModeIndicator />
  </header>

  <div class="content">
    <aside class="sidebar">
      <DeviceScanner />
      <hr />
      <PassthroughControls />
    </aside>

    <section class="main-area">
      <VideoFeed />
    </section>
  </div>
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    background: #0f0f0f;
    color: #e0e0e0;
    overflow: hidden;
  }
  :global(*) {
    box-sizing: border-box;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
  }

  .top-bar {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem 1rem;
    background: #1a1a1a;
    border-bottom: 1px solid #2a2a2a;
    flex-shrink: 0;
  }

  .brand {
    font-weight: 700;
    font-size: 1.1rem;
    color: #3b82f6;
    flex-shrink: 0;
  }

  .content {
    display: flex;
    flex: 1;
    min-height: 0;
  }

  .sidebar {
    width: 280px;
    flex-shrink: 0;
    background: #141414;
    border-right: 1px solid #2a2a2a;
    overflow-y: auto;
  }

  .sidebar hr {
    border: none;
    border-top: 1px solid #2a2a2a;
    margin: 0;
  }

  .main-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
</style>
