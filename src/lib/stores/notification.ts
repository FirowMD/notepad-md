import { writable } from 'svelte/store';

interface Notification {
  id: string;
  message: string;
  type: 'success' | 'error' | 'info';
  duration: number;
}

interface NotificationState {
  notifications: Notification[];
}

function createNotificationStore() {
  const { subscribe, update } = writable<NotificationState>({
    notifications: []
  });

  return {
    subscribe,
    show: (message: string, type: 'success' | 'error' | 'info' = 'info', duration: number = 3000) => {
      const id = Math.random().toString(36).substr(2, 9);
      const notification: Notification = { id, message, type, duration };
      
      update(state => ({
        notifications: [...state.notifications, notification]
      }));

      // Auto-remove notification after duration
      setTimeout(() => {
        update(state => ({
          notifications: state.notifications.filter(n => n.id !== id)
        }));
      }, duration);
    },
    remove: (id: string) => {
      update(state => ({
        notifications: state.notifications.filter(n => n.id !== id)
      }));
    },
    clear: () => {
      update(() => ({ notifications: [] }));
    }
  };
}

export const notificationStore = createNotificationStore();