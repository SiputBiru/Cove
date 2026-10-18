type Gsap = typeof import('gsap').gsap;
type QuickTo = ReturnType<Gsap['quickTo']>;

type MagneticOptions = {
	strength?: number;
	max?: number;
	duration?: number;
	ease?: string;
};

let gsapPromise: Promise<Gsap> | undefined;

function loadGsap(): Promise<Gsap> {
	gsapPromise ??= import('gsap').then((m) => m.gsap);
	return gsapPromise;
}

export function magnetic(
	node: HTMLElement,
	{ strength = 0.075, max = 12, duration = 0.4, ease = 'power2.out' }: MagneticOptions = {}
) {
	const zone = node.parentElement;
	if (!zone) return {};

	let gsap: Gsap | undefined;
	let xTo: QuickTo | undefined;
	let yTo: QuickTo | undefined;
	let destroyed = false;

	let loading = false;
	
	const onEnter = () => {
		if (gsap || loading) return;
		loading = true;
		loadGsap().then((g) => {
			if (destroyed) return;
			gsap = g;
			xTo = g.quickTo(node, 'x', { duration, ease });
			yTo = g.quickTo(node, 'y', { duration, ease });
		});
	};

	const onMove = (e: PointerEvent) => {
		if (!xTo || !yTo) return;

		const rect = zone.getBoundingClientRect();
		const dx = e.clientX - (rect.left + rect.width / 2);
		const dy = e.clientY - (rect.top + rect.height / 2);

		const targetX = Math.max(-max, Math.min(max, dx * strength));
		const targetY = Math.max(-max, Math.min(max, dy * strength));

		xTo(targetX);
		yTo(targetY);
	};

	const onLeave = () => {
		xTo?.(0);
		yTo?.(0);
	};

	zone.addEventListener('pointerenter', onEnter);
	zone.addEventListener('pointermove', onMove);
	zone.addEventListener('pointerleave', onLeave);

	return {
		destroy() {
			destroyed = true;
			zone.removeEventListener('pointerenter', onEnter);
			zone.removeEventListener('pointermove', onMove);
			zone.removeEventListener('pointerleave', onLeave);
			gsap?.killTweensOf(node);
		}
	};
}