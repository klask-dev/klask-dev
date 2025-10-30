import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import { getApiBaseUrl } from './lib/config'

// Set API base URL on window object for use in fetch requests
window.API_BASE_URL = getApiBaseUrl()

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
