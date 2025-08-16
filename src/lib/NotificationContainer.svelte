<script lang="ts">
  import { notificationStore } from './stores/notification';
  import NotificationToast from './NotificationToast.svelte';
  import { flip } from 'svelte/animate';

  function handleClose(event: CustomEvent<string>) {
    notificationStore.remove(event.detail);
  }
</script>

{#if $notificationStore.notifications.length > 0}
  <div class="fixed top-4 right-4 z-50 flex flex-col gap-3 max-w-md pointer-events-none">
    {#each $notificationStore.notifications as notification (notification.id)}
      <div 
        class="pointer-events-auto"
        animate:flip={{ duration: 300 }}
      >
        <NotificationToast 
          {notification} 
          on:close={handleClose}
        />
      </div>
    {/each}
  </div>
{/if}