type MagneticOptions = {
  strength?: number;
  max?: number;
  ease?: number;
};

export function magnetic(
  node: HTMLElement,
  { strength = 0.1, max = 12, ease = 0.15 }: MagneticOptions = {},
) {
  const zone = node.parentElement;
  if (!zone) return {};

  let x = 0;
  let y = 0;
  let targetX = 0;
  let targetY = 0;
  let raf = 0;

  const tick = () => {
    x += (targetX - x) * ease;
    y += (targetY - y) * ease;
    node.style.translate = `${x}px ${y}px`;

    const settled =
      Math.abs(targetX - x) < 0.05 && Math.abs(targetY - y) < 0.05;
    if (settled) {
      x = targetX;
      y = targetY;
      node.style.translate = `${x}px ${y}px`;
      raf = 0;
      return;
    }
    raf = requestAnimationFrame(tick);
  };

  const run = () => {
    if (!raf) raf = requestAnimationFrame(tick);
  };

  const onMove = (e: PointerEvent) => {
    const rect = zone.getBoundingClientRect();
    const dx = e.clientX - (rect.left + rect.width / 2);
    const dy = e.clientY - (rect.top + rect.height / 2);
    targetX = Math.max(-max, Math.min(max, dx * strength));
    targetY = Math.max(-max, Math.min(max, dy * strength));
    run();
  };

  const onLeave = () => {
    targetX = 0;
    targetY = 0;
    run();
  };

  zone.addEventListener("pointermove", onMove);
  zone.addEventListener("pointerleave", onLeave);

  return {
    destroy() {
      zone.removeEventListener("pointermove", onMove);
      zone.removeEventListener("pointerleave", onLeave);
      cancelAnimationFrame(raf);
    },
  };
}
