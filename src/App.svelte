<script lang="ts">
  let open = false;
</script>

<main>
  <button class="island" class:open on:click={() => (open = !open)}>
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
    transition: transform 0.3s ease;
    transform-origin: top center;
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
  .island.open { transform: scale(1.1); }
</style>
