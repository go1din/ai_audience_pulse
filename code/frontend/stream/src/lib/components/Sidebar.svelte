<script lang="ts">
  export let isOpen = false;
  
  function toggleSidebar() {
    isOpen = !isOpen;
  }
</script>

<style>
  .sidebar-toggle {
    position: fixed;
    left: 1rem;
    top: 50%;
    transform: translateY(-50%);
    width: 50px;
    height: 50px;
    background: rgba(52, 211, 153, 0.15);
    border: 1.5px solid rgba(52, 211, 153, 0.4);
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    z-index: 99;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    backdrop-filter: blur(16px) saturate(180%);
  }

  .sidebar-toggle:hover {
    background: rgba(52, 211, 153, 0.25);
    transform: translateY(-50%) scale(1.05);
  }

  .sidebar-toggle-icon {
    font-size: 1.5rem;
  }

  .sidebar {
    position: fixed;
    left: 0;
    top: 70px;
    bottom: 50px;
    width: 280px;
    background: rgba(10, 14, 39, 0.98);
    backdrop-filter: blur(20px) saturate(180%);
    border-right: 1px solid rgba(52, 211, 153, 0.2);
    transform: translateX(-100%);
    transition: transform 0.4s cubic-bezier(0.4, 0, 0.2, 1);
    z-index: 98;
    box-shadow: 4px 0 20px rgba(0, 0, 0, 0.3);
    overflow-y: auto;
  }

  .sidebar.open {
    transform: translateX(0);
  }

  .sidebar-content {
    padding: 2rem 1.5rem;
  }

  .sidebar-section {
    margin-bottom: 2rem;
  }

  .sidebar-title {
    font-size: 0.875rem;
    font-weight: 700;
    color: rgba(255, 255, 255, 0.5);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    margin-bottom: 1rem;
  }

  .sidebar-item {
    padding: 0.875rem 1rem;
    border-radius: 0.75rem;
    color: rgba(255, 255, 255, 0.8);
    cursor: pointer;
    transition: all 0.3s ease;
    margin-bottom: 0.5rem;
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .sidebar-item:hover {
    background: rgba(52, 211, 153, 0.1);
    color: rgba(52, 211, 153, 1);
  }

  .sidebar-icon {
    font-size: 1.25rem;
  }

  @media (max-width: 768px) {
    .sidebar {
      top: 60px;
      bottom: 45px;
      width: 240px;
    }
  }
</style>

<button class="sidebar-toggle" on:click={toggleSidebar}>
  <span class="sidebar-toggle-icon">{isOpen ? '✕' : '☰'}</span>
</button>

<aside class="sidebar" class:open={isOpen}>
  <div class="sidebar-content">
    <div class="sidebar-section">
      <h3 class="sidebar-title">Quick Actions</h3>
      <div class="sidebar-item">
        <span class="sidebar-icon">📊</span>
        <span>Analytics</span>
      </div>
      <div class="sidebar-item">
        <span class="sidebar-icon">⚙️</span>
        <span>Settings</span>
      </div>
      <div class="sidebar-item">
        <span class="sidebar-icon">📝</span>
        <span>Reports</span>
      </div>
    </div>

    <div class="sidebar-section">
      <h3 class="sidebar-title">Tools</h3>
      <div class="sidebar-item">
        <span class="sidebar-icon">🎨</span>
        <span>Themes</span>
      </div>
      <div class="sidebar-item">
        <span class="sidebar-icon">🔔</span>
        <span>Notifications</span>
      </div>
    </div>
  </div>
</aside>
