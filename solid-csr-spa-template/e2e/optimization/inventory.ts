import ts from "typescript";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import type { Actor } from "./model";

export interface PageRoute { pattern: string; actor: Actor }
/** Read route declarations without loading Solid components or trusting the prose map. */
export function routes(source: string): PageRoute[] {
  const file = ts.createSourceFile("routes.ts", source, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS);
  const result = new Map<string, PageRoute>();
  function visit(node: ts.Node, parent = "", inherited: Actor = "anonymous"): void {
    if (!ts.isObjectLiteralExpression(node)) { ts.forEachChild(node, (child) => visit(child, parent, inherited)); return; }
    const property = (name: string) => node.properties.find((p) => ts.isPropertyAssignment(p) && p.name.getText(file) === name);
    const path = property("path");
    if (!path || !ts.isPropertyAssignment(path) || !ts.isStringLiteral(path.initializer)) return;
    const local = path.initializer.text;
    const pattern = local.startsWith("*") ? local : `${parent}${local}`.replace(/\/$/, "") || "/";
    const component = property("component")?.getText(file) ?? "";
    const actor: Actor = component.includes("withSuperuser(") ? "admin" : component.includes("withAuth(") ? "member" : inherited;
    const previous = result.get(pattern);
    if (!previous || previous.actor === "anonymous") result.set(pattern, { pattern, actor });
    const children = property("children");
    if (children && ts.isPropertyAssignment(children)) ts.forEachChild(children.initializer, (child) => visit(child, pattern === "/" ? "" : pattern, actor));
  }
  visit(file);
  if (result.size === 0) throw new Error("Could not derive browser route inventory");
  return [...result.values()];
}
export function currentRoutes(): PageRoute[] {
  return routes(readFileSync(resolve(import.meta.dirname, "../../src/routes.ts"), "utf8"));
}
export function pagePath(pattern: string, parameters: Record<string, string>): string {
  if (pattern === "*404") return "/optimization-unmatched-page";
  return pattern.replace(/:([A-Za-z_][A-Za-z0-9_]*)/g, (_match: string, key: string) => {
    const value = parameters[key];
    if (!value || value.startsWith("REPLACE_")) throw new Error(`Missing page fixture ${key}`);
    return encodeURIComponent(value);
  });
}
