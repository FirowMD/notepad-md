<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fly, scale } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';

  export let notification: {
    id: string;
    message: string;
    type: 'success' | 'error' | 'info';
    duration: number;
  };

  const dispatch = createEventDispatcher();

  function handleClose() {
    dispatch('close', notification.id);
  }

  $: bgClass = notification.type === 'success' ? 'bg-gradient-to-r from-tertiary-500/10 to-tertiary-600/5' :
               notification.type === 'error' ? 'bg-gradient-to-r from-error-500/10 to-error-600/5' :
               'bg-gradient-to-r from-primary-500/10 to-primary-600/5';
  
  $: borderClass = notification.type === 'success' ? 'border-tertiary-500/30' :
                   notification.type === 'error' ? 'border-error-500/30' :
                   'border-primary-500/30';
  
  $: textClass = notification.type === 'success' ? 'text-tertiary-100' :
                 notification.type === 'error' ? 'text-error-100' :
                 'text-primary-100';
  
  $: iconClass = notification.type === 'success' ? 'bg-tertiary-500' :
                 notification.type === 'error' ? 'bg-error-500' :
                 'bg-primary-500';
  
  $: ringClass = notification.type === 'success' ? 'focus:ring-tertiary-500/50' :
                 notification.type === 'error' ? 'focus:ring-error-500/50' :
                 'focus:ring-primary-500/50';
</script>

<div 
  class="preset-glass relative overflow-hidden rounded-lg border shadow-xl max-w-sm {bgClass} {borderClass}"
  in:fly={{ x: 300, duration: 400, easing: quintOut }}
  out:fly={{ x: 300, duration: 300, easing: quintOut }}
>
  <div class="flex items-start gap-3 p-4">
    <div 
      class="flex-shrink-0 w-6 h-6 rounded-full {iconClass} flex items-center justify-center shadow-md"
      in:scale={{ duration: 300, delay: 200, easing: quintOut }}
    >
      {#if notification.type === 'success'}
        <svg class="w-3.5 h-3.5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2.5">
          <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7"></path>
        </svg>
      {:else if notification.type === 'error'}
        <svg class="w-3.5 h-3.5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2.5">
          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12"></path>
        </svg>
      {:else}
        <svg class="w-3.5 h-3.5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2.5">
          <path stroke-linecap="round" stroke-linejoin="round" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
        </svg>
      {/if}
    </div>
    
    <div class="flex-1 min-w-0 pt-0.5">
      <p class="text-sm font-medium {textClass} leading-5 pr-2">
        {notification.message}
      </p>
    </div>
    
    <button
      type="button"
      class="flex-shrink-0 rounded-md p-1.5 {textClass} hover:bg-surface-600/50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-transparent {ringClass} transition-all duration-200"
      onclick={handleClose}
    >
      <span class="sr-only">Dismiss</span>
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12"></path>
      </svg>
    </button>
  </div>
</div>
