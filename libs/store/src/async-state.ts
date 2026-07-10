import type { ClassifiedError } from "@asha-rulebench/protocol";

export type AsyncState<T, E = ClassifiedError> =
  | { readonly kind: "idle" }
  | { readonly kind: "loading" }
  | { readonly kind: "data"; readonly value: T }
  | { readonly kind: "error"; readonly error: E };
