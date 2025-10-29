/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_API_BASE_URL: string
  readonly VITE_APP_TITLE: string
  readonly DEV: boolean
  readonly PROD: boolean
  readonly MODE: string
  // add more env variables as needed
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}

// Add process global for Node.js compatibility
declare let process: {
  env: {
    NODE_ENV?: 'development' | 'production' | 'test'
  }
} | undefined

// Declare react-dom/client module
declare module 'react-dom/client' {
  import { Container } from 'react-dom'
  export interface Root {
    render(children: React.ReactNode): void
    unmount(): void
  }
  export function createRoot(container: Container): Root
}

// Runtime configuration injected by Docker entrypoint
declare global {
  interface Window {
    RUNTIME_CONFIG?: {
      VITE_API_BASE_URL?: string;
    };
    API_BASE_URL?: string;
  }
}

export {};
