import { Err, Ok, safeAsync, type Result } from 'ripthrow';
import type { z } from 'zod';

export type FetchError =
	| {
			kind: 'NetworkError';
			message: string;
			cause: unknown;
	  }
	| {
			kind: 'HttpError';
			message: string;
			status: number;
			statusText: string;
			body: string | null;
	  }
	| {
			kind: 'JsonParseError';
			message: string;
			cause: unknown;
	  }
	| {
			kind: 'ValidationError';
			message: string;
			issues: z.ZodIssue[];
	  };

export type FetchOptions = RequestInit & {
	parseJson?: boolean;
};

async function readErrorBody(response: Response): Promise<string | null> {
	const result = await safeAsync(response.text());

	if (!result.ok) {
		return null;
	}

	return result.value;
}

export async function safeFetch(
	input: RequestInfo | URL,
	init?: RequestInit
): Promise<Result<Response, FetchError>> {
	const result = await safeAsync(fetch(input, init));

	if (!result.ok) {
		return Err({
			kind: 'NetworkError',
			message: 'The request failed before receiving a response',
			cause: result.error
		});
	}

	const response = result.value;
	if (!response.ok) {
		return Err({
			kind: 'HttpError',
			message: `HTTP ${response.status} ${response.statusText}`,
			status: response.status,
			statusText: response.statusText,
			body: await readErrorBody(response)
		});
	}

	return Ok(response);
}

export async function safeFetchJson<T>(
	input: RequestInfo | URL,
	schema: z.ZodType<T>,
	init?: FetchOptions
): Promise<Result<T, FetchError>> {
	const responseResult = await safeFetch(input, init);

	if (!responseResult.ok) {
		return responseResult;
	}

	const jsonResult = await safeAsync(responseResult.value.json());

	if (!jsonResult.ok) {
		return Err({
			kind: 'JsonParseError',
			message: 'The response body was not valid JSON.',
			cause: jsonResult.error
		});
	}

	const parsed = schema.safeParse(jsonResult.value);

	if (!parsed.success) {
		return Err({
			kind: 'ValidationError',
			message: 'The response JSON did not match the expected schema.',
			issues: parsed.error.issues
		});
	}

	return Ok(parsed.data);
}
