<script lang="ts">
  export let message: string;
  export let type: 'error' | 'warning' | 'info' = 'error';
</script>

<style>
  .alert {
    padding: 1rem;
    border-radius: 0.5rem;
    margin: 1rem 0;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    animation: slideIn 0.3s ease-out;
  }

  .error {
    background-color: rgba(239, 68, 68, 0.1);
    border: 1px solid rgb(239, 68, 68);
    color: rgb(185, 28, 28);
  }

  .warning {
    background-color: rgba(245, 158, 11, 0.1);
    border: 1px solid rgb(245, 158, 11);
    color: rgb(180, 83, 9);
  }

  .info {
    background-color: rgba(59, 130, 246, 0.1);
    border: 1px solid rgb(59, 130, 246);
    color: rgb(29, 78, 216);
  }

  @keyframes slideIn {
    from {
      transform: translateY(-1rem);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }
</style>

<div class="alert {type}" role="alert">
  {#if type === 'error'}
    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
    </svg>
  {:else if type === 'warning'}
    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
    </svg>
  {:else}
    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
    </svg>
  {/if}
  <span>{message}</span>
</div>