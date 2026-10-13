<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  let open = false;

  async function toggle() {
    if (!open) {
      await invoke("resize_island", { width: 360, height: 120 });
      open = true;
    } else {
      open = false;
      setTimeout(() => invoke("resize_island", { width: 220, height: 44 }), 350);
    }
  }
</script>

<main>
  <button class="island" class:open on:click={toggle}>
    {open ? "Expanded" : ""}
  </button>
</main>

<style>
  main { display: flex; justify-content: center; }
  .island {
    --ear: 16px;
    position: relative;
    width: 188px; height: 36px;
    border: 0; border-radius: 0 0 16px 16px;
    background: #000; color: #fff;
    transition: width 0.3s ease, height 0.3s ease, border-radius 0.3s ease;
  }
  .island::before, .island::after {
    content: "";
    position: absolute; top: 0;
    width: var(--ear); height: var(--ear);
  }
  .island::before {
    left: calc(var(--ear) * -1);
    background: radial-gradient(circle at 0 100%, transparent calc(var(--ear) - 0.2px), #000 var(--ear));
  }
  .island::after {
    right: calc(var(--ear) * -1);
    background: radial-gradient(circle at 100% 100%, transparent calc(var(--ear) - 0.2px), #000 var(--ear));
  }
  .island.open { width: 328px; height: 100px; border-radius: 0 0 28px 28px; }
</style>