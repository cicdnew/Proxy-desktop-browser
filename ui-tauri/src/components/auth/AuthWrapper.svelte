<script lang="ts">
  import { setContext } from 'svelte';
  import { writable } from 'svelte/store';

  export interface AuthContext {
    isAuthenticated: boolean;
    user: any | null;
    login: (credentials: any) => Promise<void>;
    logout: () => void;
  }

  // Create a stub auth context
  const authContext: AuthContext = {
    isAuthenticated: true,
    user: { id: 'demo', name: 'Demo User' },
    login: async () => {},
    logout: () => {}
  };

  setContext('auth', authContext);
</script>

<slot />
