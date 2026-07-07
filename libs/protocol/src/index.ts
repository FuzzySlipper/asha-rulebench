export * from './generated/api-types';

export type ClassifiedError =
  | { readonly kind: 'network'; readonly message: string; readonly retryable: true }
  | { readonly kind: 'unauthorized'; readonly message: string; readonly retryable: false }
  | { readonly kind: 'not-found'; readonly message: string; readonly retryable: false }
  | { readonly kind: 'unknown'; readonly message: string; readonly retryable: false };

export type Result<T> = { readonly ok: true; readonly value: T } | { readonly ok: false; readonly error: ClassifiedError };

export const unknownError = (message: string): ClassifiedError => ({
  kind: 'unknown',
  message,
  retryable: false,
});
