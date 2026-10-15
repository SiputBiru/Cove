import gsap from 'gsap';

type MagneticOptions = {
	strength?: number;
	max?: number;
	duration?: number;
	ease?: string;
};

export function magnetic(
	node: HTMLElement,
	{ strength = 0.075, max = 12, duration = 0.4, ease = 'power2.out' }: MagneticOptions = {}
) {
	const zone = node.parentElement;
	if (!zone) return {};

	const xTo = gsap.quickTo(node, 'x', { duration, ease });
	const yTo = gsap.quickTo(node, 'y', { duration, ease });

	const onMove = (e: PointerEvent) => {
		const rect = zone.getBoundingClientRect();
		const dx = e.clientX - (rect.left + rect.width / 2);
		const dy = e.clientY - (rect.top + rect.height / 2);

		const targetX = Math.max(-max, Math.min(max, dx * strength));
		const targetY = Math.max(-max, Math.min(max, dy * strength));

		xTo(targetX);
		yTo(targetY);
	};

	const onLeave = () => {
		xTo(0);
		yTo(0);
	};

	zone.addEventListener('pointermove', onMove);
	zone.addEventListener('pointerleave', onLeave);

	return {
		destroy() {
			zone.removeEventListener('pointermove', onMove);
			zone.removeEventListener('pointerleave', onLeave);
			gsap.killTweensOf(node);
		}
	};
}