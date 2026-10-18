import { derived, writable } from "svelte/store";

export type Task = { id: string; text: string; done: boolean };

const STORAGE_KEY = "cove.todos";

function load(): Task[] {
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        const parsed = raw ? JSON.parse(raw) : [];
        if (!Array.isArray(parsed)) return [];
        return parsed.filter(
            (t: any): t is Task =>
                typeof t?.id === "string" &&
                typeof t?.text === "string" &&
                typeof t?.done === "boolean",
        );
    } catch {
        return [];
    }
}

export const todos = writable<Task[]>(load());

todos.subscribe((list) => {
    try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(list));
    } catch {
    }
});

export const remaining = derived(
    todos,
    ($todos) => $todos.filter((t) => !t.done).length,
);

const newId = () =>
    Date.now().toString(36) + Math.random().toString(36).slice(2, 6);

export function addTask(text: string): boolean {
    const clean = text.trim();
    if (!clean) return false;
    todos.update((list) => [{ id: newId(), text: clean, done: false }, ...list]);
    return true;
}

export function toggleTask(id: string) {
    todos.update((list) =>
        list.map((t) => (t.id === id ? { ...t, done: !t.done } : t)),
    );
}

export function removeTask(id: string) {
    todos.update((list) => list.filter((t) => t.id !== id));
}

export function clearDone() {
    todos.update((list) => list.filter((t) => !t.done));
}
