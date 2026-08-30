// Keyed record stores with (key, value) setters. Solid 2.0 store setters are
// draft-mutation only; this wraps the common per-id flag/text pattern so call
// sites stay terse.

import { createStore, type Store } from "solid-js";

export function createKeyedStore<T>(): [
  Store<Record<string, T>>,
  (key: string, value: T) => void,
] {
  const [store, setStore] = createStore<Record<string, T>>({});
  const set = (key: string, value: T) =>
    setStore((s) => {
      s[key] = value;
    });
  return [store, set];
}
