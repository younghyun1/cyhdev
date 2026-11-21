import { isAuthenticated, isSuperuser } from "../../state/auth";

/**
 * Whether the archive may be shown to this visitor. Entry points (the Projects
 * card, the admin navigation) use this; the server and the route guards remain
 * authoritative.
 */
export const canViewArchive = (): boolean => isAuthenticated() === true && isSuperuser() === true;
