<script lang="ts">
  import Header from '$lib/components/Header.svelte';
  import Footer from '$lib/components/Footer.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  
  let sidebarOpen = false;
</script>

<style>
  .page-container {
    position: fixed;
    top: 70px;
    left: 0;
    right: 0;
    bottom: 50px;
    background: linear-gradient(135deg, #0a0e27 0%, #1a1f3a 50%, #0f1729 100%);
  }

  .iframe-container {
    width: 100%;
    height: 100%;
    position: relative;
  }

  .iframe-wrapper {
    width: 100%;
    height: 100%;
    border: none;
    background: #fff;
  }

  .loading-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(10, 14, 39, 0.95);
    color: #fff;
    font-size: 1.25rem;
    z-index: 1;
  }

  @media (max-width: 768px) {
    .page-container {
      top: 60px;
      bottom: 45px;
    }
  }
</style>

<Header currentPage="thumbs" />
<Sidebar bind:isOpen={sidebarOpen} />

<div class="page-container">
  <div class="iframe-container">
    <iframe 
      src="https://thumbs-up.surge.sh/" 
      class="iframe-wrapper"
      title="Thumbs Up Application"
      allow="camera; microphone"
    ></iframe>
  </div>
</div>

<Footer />
