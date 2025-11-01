<script lang="ts">
  export let isVisible = false;
  export let x = 0;
  export let y = 0;

  let randomOffset = Math.random() * 10 - 5;
  let randomDelay = Math.random() * 0.5;
</script>

<style>
  .silence-container {
    position: absolute;
    pointer-events: none;
    display: flex;
    flex-direction: column;
    align-items: center;
    opacity: 0;
    transform: scale(0.8);
    transition: opacity 0.3s ease-in-out, transform 0.3s ease-out;
  }

  .silence-container.visible {
    opacity: 1;
    transform: scale(1);
  }

  .silence-banner {
    background: linear-gradient(90deg, 
      rgba(0, 0, 0, 0) 0%,
      rgba(66, 153, 225, 0.15) 20%,
      rgba(66, 153, 225, 0.15) 80%,
      rgba(0, 0, 0, 0) 100%
    );
    padding: 1rem 2rem;
    border-radius: 1rem;
    backdrop-filter: blur(4px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
    display: flex;
    align-items: center;
    gap: 1rem;
    animation: glow 2s ease-in-out infinite;
  }

  .emoji-container {
    position: relative;
    font-size: 2rem;
    animation: sleepBounce 4s ease-in-out infinite;
  }

  .silence-text {
    font-family: system-ui, -apple-system, sans-serif;
    font-size: 1.1rem;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.9);
    text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
    letter-spacing: 0.05em;
  }

  .zzz-container {
    position: absolute;
    top: -1rem;
    right: -1rem;
    transform-origin: bottom left;
  }

  .zzz {
    position: absolute;
    font-family: 'Comic Sans MS', cursive;
    font-weight: bold;
    color: rgba(255, 255, 255, 0.8);
    text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  }

  .zzz:nth-child(1) { 
    font-size: 0.8rem;
    animation: floatZzz 2s ease-in-out infinite;
    animation-delay: 0s;
  }
  .zzz:nth-child(2) { 
    font-size: 1rem;
    animation: floatZzz 2s ease-in-out infinite;
    animation-delay: 0.3s;
  }
  .zzz:nth-child(3) { 
    font-size: 1.2rem;
    animation: floatZzz 2s ease-in-out infinite;
    animation-delay: 0.6s;
  }

  .silence-waves {
    margin-top: 0.5rem;
    height: 2px;
    width: 100%;
    background: linear-gradient(90deg,
      rgba(66, 153, 225, 0) 0%,
      rgba(66, 153, 225, 0.3) 50%,
      rgba(66, 153, 225, 0) 100%
    );
    animation: waveScale 4s ease-in-out infinite;
  }

  @keyframes glow {
    0%, 100% { box-shadow: 0 4px 12px rgba(66, 153, 225, 0.1); }
    50% { box-shadow: 0 4px 20px rgba(66, 153, 225, 0.2); }
  }

  @keyframes sleepBounce {
    0%, 100% { transform: translateY(0) rotate(0deg); }
    25% { transform: translateY(-2px) rotate(-5deg); }
    75% { transform: translateY(2px) rotate(5deg); }
  }

  @keyframes floatZzz {
    0% {
      opacity: 0;
      transform: translate(0, 0) scale(0.8);
    }
    50% {
      opacity: 1;
      transform: translate(-15px, -15px) scale(1);
    }
    100% {
      opacity: 0;
      transform: translate(-30px, -30px) scale(0.8);
    }
  }

  @keyframes waveScale {
    0%, 100% { transform: scaleY(1); opacity: 0.3; }
    50% { transform: scaleY(2); opacity: 0.6; }
  }
</style>

<div 
  class="silence-container"
  class:visible={isVisible}
  style="left: {x}px; top: {y}px;"
>
  <div class="silence-banner">
    <div class="emoji-container">
      😴
      <div class="zzz-container">
        <span class="zzz">z</span>
        <span class="zzz">z</span>
        <span class="zzz">z</span>
      </div>
    </div>
    <span class="silence-text">Silence detected</span>
  </div>
  <div class="silence-waves"></div>
</div>