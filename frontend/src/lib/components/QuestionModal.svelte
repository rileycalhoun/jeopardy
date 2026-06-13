<script lang="ts">
	import type { Snippet } from 'svelte';

	import type { GameView } from '$lib/api/schemas';

	type ActiveClue = NonNullable<GameView['active_clue']>;

	let {
		open,
		clue,
		categoryTitle = '',
		showAnswer = false,
		dismissable = false,
		onClose,
		children
	}: {
		/** When true the modal takes over the screen; when false nothing renders. */
		open: boolean;
		clue: ActiveClue;
		categoryTitle?: string;
		/** Hosts see the expected answer; players never do. */
		showAnswer?: boolean;
		/**
		 * Normal players cannot cancel a clue, so the modal stays put until the
		 * clue resolves. Set true to allow Escape / backdrop / close-button dismissal.
		 */
		dismissable?: boolean;
		onClose?: () => void;
		/** Controls under the clue (answer form for players, host controls for hosts). */
		children?: Snippet;
	} = $props();

	// Unique ids so the dialog can point screen readers at its title and question.
	const titleId = `question-modal-title-${Math.random().toString(36).slice(2)}`;
	const questionId = `question-modal-question-${Math.random().toString(36).slice(2)}`;

	let dialogEl = $state<HTMLDivElement | null>(null);

	const FOCUSABLE =
		'a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])';

	function focusable(): HTMLElement[] {
		if (!dialogEl) return [];
		return Array.from(dialogEl.querySelectorAll<HTMLElement>(FOCUSABLE));
	}

	// When the modal opens, move focus inside (preferring the marked answer input),
	// lock the background from scrolling, and restore both when it closes.
	$effect(() => {
		if (!open || !dialogEl) return;

		const previouslyFocused = document.activeElement as HTMLElement | null;
		const target =
			dialogEl.querySelector<HTMLElement>('[data-autofocus]') ?? focusable()[0] ?? dialogEl;
		// Defer so the element is laid out before we focus it.
		queueMicrotask(() => target.focus());

		const previousOverflow = document.body.style.overflow;
		document.body.style.overflow = 'hidden';

		return () => {
			document.body.style.overflow = previousOverflow;
			previouslyFocused?.focus?.();
		};
	});

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape' && dismissable) {
			event.preventDefault();
			onClose?.();
			return;
		}

		if (event.key !== 'Tab') return;

		// Keep keyboard focus cycling within the dialog.
		const items = focusable();
		if (items.length === 0) {
			event.preventDefault();
			dialogEl?.focus();
			return;
		}

		const first = items[0];
		const last = items[items.length - 1];
		if (event.shiftKey && document.activeElement === first) {
			event.preventDefault();
			last.focus();
		} else if (!event.shiftKey && document.activeElement === last) {
			event.preventDefault();
			first.focus();
		}
	}

	function handleBackdrop() {
		if (dismissable) onClose?.();
	}
</script>

{#if open}
	<!-- The dimmed backdrop covers and disables the board behind the clue. -->
	<div class="fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-6">
		<button
			type="button"
			aria-label="Close question"
			tabindex="-1"
			class="absolute inset-0 bg-black/75 backdrop-blur-sm"
			class:cursor-pointer={dismissable}
			class:cursor-default={!dismissable}
			onclick={handleBackdrop}
		></button>

		<!-- The clue itself: zoomed-in like a board tile taking over the TV screen. -->
		<div
			bind:this={dialogEl}
			role="dialog"
			aria-modal="true"
			aria-labelledby={titleId}
			aria-describedby={questionId}
			tabindex="-1"
			onkeydown={handleKeydown}
			class="relative z-10 flex max-h-[90vh] w-full max-w-3xl flex-col overflow-y-auto rounded-lg border-4 border-black/70 bg-gradient-to-b from-board-dark to-board p-6 text-center shadow-2xl shadow-black/60 outline-none sm:p-8"
		>
			{#if dismissable}
				<button
					type="button"
					aria-label="Close"
					class="absolute top-3 right-4 text-2xl leading-none text-white/60 transition hover:text-white"
					onclick={() => onClose?.()}
				>
					&times;
				</button>
			{/if}

			<p id={titleId} class="show-eyebrow">
				{#if categoryTitle}{categoryTitle} ·{/if}
				{clue.label}
			</p>

			<h2
				id={questionId}
				class="mx-auto mt-5 max-w-3xl font-display text-2xl leading-snug font-bold text-white uppercase [text-shadow:2px_2px_4px_rgba(0,0,0,0.8)] sm:text-3xl md:text-4xl"
			>
				{clue.question}
			</h2>

			{#if showAnswer && clue.answer}
				<p class="mt-5 text-lg text-gold-soft">
					Answer: <span class="font-semibold">{clue.answer}</span>
				</p>
			{/if}

			{#if children}
				<div class="mt-6 text-left">
					{@render children()}
				</div>
			{/if}
		</div>
	</div>
{/if}
