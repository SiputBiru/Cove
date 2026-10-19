import { mount } from 'svelte'
import './app.css'
import './macos.css'
import App from './App.svelte'
import { platform } from './lib/platform'

const root = document.documentElement
root.dataset.platform = platform

const app = mount(App, {
  target: document.getElementById('app')!,
})

export default app
