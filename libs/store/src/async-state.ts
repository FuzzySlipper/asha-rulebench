import type { ClassifiedError } from "@asha-rulebench/protocol";

export type AsyncState<T> =
  | { readonly kind: "idle" }
  | { readonly kind: "loading" }
  | { readonly kind: "data"; readonly value: T }
  | { readonly kind: "error"; readonly error: ClassifiedError };
