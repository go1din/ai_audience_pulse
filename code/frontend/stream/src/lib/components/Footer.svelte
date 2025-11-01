<style>
  .footer {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    height: 50px;
    background: rgba(10, 14, 39, 0.95);
    backdrop-filter: blur(20px) saturate(180%);
    border-top: 1px solid rgba(52, 211, 153, 0.2);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    box-shadow: 0 -4px 20px rgba(0, 0, 0, 0.3);
  }

  .footer-content {
    display: flex;
    align-items: center;
    gap: 1.5rem;
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.6);
  }

  .footer-link {
    color: rgba(52, 211, 153, 0.8);
    text-decoration: none;
    transition: color 0.3s ease;
  }

  .footer-link:hover {
    color: rgba(52, 211, 153, 1);
  }

  @media (max-width: 768px) {
    .footer {
      height: 45px;
    }

    .footer-content {
      font-size: 0.75rem;
      gap: 1rem;
    }
  }
</style>

<footer class="footer">
  <div class="footer-content">
    <span>© 2025 AI Audience Pulse</span>
    <span>•</span>
    <a href="#" class="footer-link">Privacy</a>
    <span>•</span>
    <a href="#" class="footer-link">Terms</a>
  </div>
</footer>
