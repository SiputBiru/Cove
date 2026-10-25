<script lang="ts">
    import { Monitor, Minimize2, ShieldCheck } from "lucide-svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    export let use24HourClock: boolean;
    export let alwaysTiny: boolean;
    export let launchAtLogin: boolean;

    let settingsSubTab: "general" | "notifications" | "about" = "general";

    const AUTOSTART_DEFAULTED_KEY = "cove:autostart-defaulted";

    onMount(async () => {
        try {
            let enabled = await invoke<boolean>("is_autostart_enabled");
            if (!enabled && !localStorage.getItem(AUTOSTART_DEFAULTED_KEY)) {
                await invoke("set_autostart", { enabled: true });
                enabled = true;
            }
            launchAtLogin = enabled;
            localStorage.setItem(AUTOSTART_DEFAULTED_KEY, "true");
        } catch {
            // ignore
        }
    });

    async function toggleAutostart() {
        const next = !launchAtLogin;
        try {
            await invoke("set_autostart", { enabled: next });
            launchAtLogin = next;
        } catch {
            launchAtLogin = !next;
        }
    }
</script>

<div class="panel">
    <div class="panel-box justify-start! p-2! gap-2">
        <!-- Settings Sub-Nav -->
        <div
            class="flex items-center justify-center gap-1.5 w-full bg-white/5 p-1 rounded-xl"
        >
            <button
                class="settings-sub-btn"
                class:active={settingsSubTab === "general"}
                on:click={() => (settingsSubTab = "general")}
            >
                General
            </button>
            <button
                class="settings-sub-btn"
                class:active={settingsSubTab === "about"}
                on:click={() => (settingsSubTab = "about")}
            >
                About
            </button>
        </div>

        <!-- Settings Content -->
        <div
            class="w-full flex-1 flex flex-col gap-1.5 overflow-y-auto pr-0.5 scrollbar-none"
        >
            {#if settingsSubTab === "general"}
                <label class="setting-row">
                    <div class="flex items-center gap-2">
                        <Monitor class="w-4 h-4 text-white/70" />
                        <span class="text-[13px]">24-Hour Format</span>
                    </div>
                    <input
                        type="checkbox"
                        bind:checked={use24HourClock}
                        class="toggle-checkbox"
                    />
                </label>

                <label class="setting-row">
                    <div class="flex items-center gap-2">
                        <Minimize2 class="w-4 h-4 text-white/70" />
                        <span class="text-[13px]">Always Tiny State</span>
                    </div>
                    <input
                        type="checkbox"
                        bind:checked={alwaysTiny}
                        class="toggle-checkbox"
                    />
                </label>

                <label class="setting-row">
                    <div class="flex items-center gap-2">
                        <ShieldCheck class="w-4 h-4 text-white/70" />
                        <span class="text-[13px]">Launch at Login</span>
                    </div>
                    <input
                        type="checkbox"
                        checked={launchAtLogin}
                        on:change={toggleAutostart}
                        class="toggle-checkbox"
                    />
                </label>
            {:else if settingsSubTab === "about"}
                <div
                    class="flex flex-col items-center justify-center flex-1 gap-0.5 py-1 text-center"
                >
                    <span class="text-[16px] font-semibold text-accent"
                        >Cove</span
                    >
                    <span class="text-[11px] font-semibold text-white/75"
                        >time, held</span
                    >
                    <span class="text-[11px] text-white/50"
                        >Version 0.6.2 beta</span
                    >

                    <!-- Credits Section -->
                    <div class="mt-0.5 text-[9px] text-white/60">
                        Handcrafted by <span class="text-accent">nobberu</span>
                    </div>
                </div>
            {/if}
        </div>
    </div>
</div>
