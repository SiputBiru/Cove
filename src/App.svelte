<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    let open = false;

    const nextFrame = () => new Promise(requestAnimationFrame);

    async function toggle() {
        if (!open) {
            await invoke("resize_island", { width: 400, height: 120 });
            await nextFrame();
            await nextFrame();
            open = true;
        } else {
            open = false;
        }
    }

    function onEnd(e: TransitionEvent) {
        if (!open && e.propertyName === "height") {
            invoke("resize_island", { width: 260, height: 80 });
        }
    }
</script>

<main>
    <button class="island" class:open on:click={toggle}>
        {open ? "Expanded" : ""}
    </button>
</main>

<style>
    main {
        display: flex;
        justify-content: center;
    }
    .island {
        --ear: 14px;
        position: relative;
        width: 188px;
        height: 30px;
        border: 0;
        border-radius: 0 0 18px 18px;
        background: #000;
        color: #fff;
        will-change: width, height, border-radius;
        transform: translateZ(0);
        transition:
            height 0.3s cubic-bezier(0.4, 0, 0.2, 1) 0s,
            width 0.3s cubic-bezier(0.4, 0, 0.2, 1) 0.3s,
            border-radius 0.3s cubic-bezier(0.4, 0, 0.2, 1) 0.3s;
    }
    .island::before,
    .island::after {
        content: "";
        position: absolute;
        top: 0;
        width: var(--ear);
        height: var(--ear);
    }
    .island::before {
        left: calc(var(--ear) * -1);
        background: radial-gradient(
            circle at 0 100%,
            transparent calc(var(--ear) - 0.5px),
            #000 var(--ear)
        );
    }
    .island::after {
        right: calc(var(--ear) * -1);
        background: radial-gradient(
            circle at 100% 100%,
            transparent calc(var(--ear) - 0.5px),
            #000 var(--ear)
        );
    }
    .island.open {
        width: 288px;
        height: 100px;
        border-radius: 0 0 28px 28px;
        transition:
            width 0.3s cubic-bezier(0.4, 0, 0.2, 1) 0s,
            height 0.3s cubic-bezier(0.4, 0, 0.2, 1) 0.3s,
            border-radius 0.3s cubic-bezier(0.4, 0, 0.2, 1) 0.3s;
    }
</style>
