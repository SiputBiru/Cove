<h1 align="center">
  <br>
  <a href="https://github.com/nobberu/cove"><img src="assets/cove.png" alt="Cove" width="150"></a>
  <br>
  Cove
  <br>
</h1>

<h3 align="center">
  Time, held.
</h3>

<br>

<p align="center">
  <a href="https://github.com/thatshubham/no-ai">
    <img src="https://img.shields.io/badge/No_AI-Generated_Content-red" alt="No AI Generated Content">
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License MIT">
  </a>
</p>

Say hello to **Cove**, a small island at the top of your screen for the things that matter about time. What time it is, what needs it, and how you're spending it. Hover for a quick glance, click to open, click again, and get right back to work.

Not a Swiss Army knife, Cove does less, on purpose. Most notch apps keep adding widgets until they become another dashboard; Cove stays minimal, light, and sheltered.

<p align="center">
  <!-- Replace with your demo asset if available -->
  <img src="assets/demo.gif" alt="Cove Demo" />
</p>

---

## Features & Highlights

- **Quiet by default:** Floats above your desktop, stays out of the taskbar, and never steals focus.
- **Glanceable:** Collapsed, it shows just the time. Hover for the date and remaining task count.
- **Light & Fast:** Built with Tauri and Svelte with optimized WebView2 memory usage.
- **Smooth Animations:** Spring-animated opening, seamlessly anchored to the top of the screen.
- **Private:** Your tasks stay saved locally on your machine.

---

## Installation

Check the [Releases](https://github.com/nobberu/cove/releases) page to download the latest package for your machine:

- **Windows:** Download `.msi` or `.exe`
- **macOS:** Download `.app.zip`
- **Linux:** Download `.AppImage` or `.deb`

---

### macOS Security Setup

Because Cove is self-signed, macOS will display an unidentified developer warning on first launch.

After moving **Cove** to your `/Applications` folder, run the following command in **Terminal** to bypass quarantine:

```bash
xattr -dr com.apple.quarantine "/Applications/Cove.app"
