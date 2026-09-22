/** Bound deletion memory to the displayed history; prevent delayed responses restoring content. */
export function rememberDeletedMessage(ids: Set<string>, id: string): void {
  ids.delete(id);
  ids.add(id);
  if (ids.size > 300) {
    const oldest = ids.values().next().value;
    if (oldest !== undefined) ids.delete(oldest);
  }
}
