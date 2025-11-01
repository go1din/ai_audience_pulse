<script lang="ts">
  import { goto } from '$app/navigation';
  
  export let currentPage: 'stream' | 'thumbs' = 'stream';
  
  function navigateToThumbs() {
    goto('/thumbs');
  }
  
  function navigateToStream() {
    goto('/');
  }
</script>

<style>
  .header {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: 70px;
    background: rgba(10, 14, 39, 0.95);
    backdrop-filter: blur(20px) saturate(180%);
    border-bottom: 1px solid rgba(52, 211, 153, 0.2);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 2rem;
    z-index: 100;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .logo-icon {
    width: 40px;
    height: 40px;
    background: linear-gradient(135deg, rgba(52, 211, 153, 0.2), rgba(16, 185, 129, 0.3));
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.5rem;
    border: 1.5px solid rgba(52, 211, 153, 0.4);
  }

  .logo-text {
    font-size: 1.25rem;
    font-weight: 700;
    color: #fff;
    letter-spacing: 0.02em;
  }

  .nav {
    display: flex;
    gap: 1rem;
  }

  .nav-button {
    padding: 0.625rem 1.5rem;
    border-radius: 2rem;
    font-size: 0.9375rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    border: 1.5px solid transparent;
    background: rgba(255, 255, 255, 0.05);
    color: rgba(255, 255, 255, 0.7);
    letter-spacing: 0.03em;
  }

  .nav-button:hover {
    background: rgba(255, 255, 255, 0.1);
    color: rgba(255, 255, 255, 0.95);
    transform: translateY(-2px);
  }

  .nav-button.active {
    background: rgba(52, 211, 153, 0.15);
    border-color: rgba(52, 211, 153, 0.4);
    color: rgba(52, 211, 153, 1);
  }

  @media (max-width: 768px) {
    .header {
      height: 60px;
      padding: 0 1rem;
    }

    .logo-text {
      font-size: 1rem;
    }

    .nav-button {
      padding: 0.5rem 1rem;
      font-size: 0.875rem;
    }
  }
</style>

<header class="header">
  <div class="logo">
    <div class="logo-icon">🎙️</div>
    <span class="logo-text">AI Audience Pulse</span>
  </div>
  
  <nav class="nav">
    <button 
      class="nav-button" 
      class:active={currentPage === 'stream'}
      on:click={navigateToStream}
    >
      Live Stream
    </button>
    <button 
      class="nav-button" 
      class:active={currentPage === 'thumbs'}
      on:click={navigateToThumbs}
    >
      Thumbs Up
    </button>
  </nav>
</header>
