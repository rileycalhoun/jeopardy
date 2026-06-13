import { userEvent } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import { expect, test, vi } from 'vitest';

import QuestionModalHarness from '$lib/test-fixtures/QuestionModalHarness.svelte';

test('renders the category, point value, and question inside a dialog', async () => {
	const screen = await render(QuestionModalHarness, { open: true });

	const dialog = screen.getByRole('dialog');
	await expect.element(dialog).toBeVisible();
	await expect.element(dialog).toHaveAttribute('aria-modal', 'true');
	await expect.element(screen.getByText('This planet is known as the Red Planet')).toBeVisible();
	await expect.element(screen.getByText(/SCIENCE/)).toBeVisible();
	await expect.element(screen.getByText(/\$400/)).toBeVisible();
});

test('renders nothing when closed', async () => {
	const screen = await render(QuestionModalHarness, { open: false });

	await expect.element(screen.getByRole('dialog')).not.toBeInTheDocument();
});

test('auto-focuses the answer input when it opens', async () => {
	const screen = await render(QuestionModalHarness, { open: true });

	await expect.element(screen.getByLabelText('Your Response')).toHaveFocus();
});

test('submitting the answer form reports the typed answer', async () => {
	const onSubmit = vi.fn();
	const screen = await render(QuestionModalHarness, { open: true, onSubmit });

	await screen.getByLabelText('Your Response').fill('Mars');
	await screen.getByRole('button', { name: 'Submit Answer' }).click();

	expect(onSubmit).toHaveBeenCalledWith('Mars');
});

test('non-dismissable modal ignores Escape', async () => {
	const onClose = vi.fn();
	const screen = await render(QuestionModalHarness, { open: true, dismissable: false, onClose });

	await screen.getByLabelText('Your Response').click();
	await userEvent.keyboard('{Escape}');

	expect(onClose).not.toHaveBeenCalled();
});

test('dismissable modal closes on Escape', async () => {
	const onClose = vi.fn();
	const screen = await render(QuestionModalHarness, { open: true, dismissable: true, onClose });

	await screen.getByLabelText('Your Response').click();
	await userEvent.keyboard('{Escape}');

	expect(onClose).toHaveBeenCalled();
});
