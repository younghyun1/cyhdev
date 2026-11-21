// Calendar arithmetic on ISO date strings, done in UTC so the local time zone
// never shifts a day.

const pad = (value: number): string => String(value).padStart(2, "0");

export const isoDate = (year: number, month: number, day: number): string =>
  `${String(year).padStart(4, "0")}-${pad(month)}-${pad(day)}`;

/** Months since year zero, for month arithmetic. */
export const monthIndex = (date: string): number =>
  Number(date.slice(0, 4)) * 12 + Number(date.slice(5, 7)) - 1;

export const monthStart = (index: number): string =>
  isoDate(Math.floor(index / 12), (index % 12) + 1, 1);

export const monthEnd = (index: number): string => {
  const year = Math.floor(index / 12);
  const month = (index % 12) + 1;
  // Day zero of the following month is the last day of this one.
  const last = new Date(Date.UTC(year, month, 0)).getUTCDate();
  return isoDate(year, month, last);
};

export interface DateJump {
  readonly from: string;
  readonly to: string;
  readonly label: string;
}

/**
 * Reads `2026`, `2026-11`, or `2026-11-21` (also with `.` or `/`) as the
 * range it names. Anything else is not a date.
 */
export const parseDateJump = (input: string): DateJump | undefined => {
  const match = /^(\d{4})(?:[-./](\d{1,2})(?:[-./](\d{1,2}))?)?\.?$/.exec(input.trim());
  if (match === null) return undefined;
  const year = Number(match[1]);
  if (year < 1990 || year > 2100) return undefined;
  if (match[2] === undefined) {
    return { from: isoDate(year, 1, 1), to: isoDate(year, 12, 31), label: String(year) };
  }
  const month = Number(match[2]);
  if (month < 1 || month > 12) return undefined;
  const start = isoDate(year, month, 1);
  if (match[3] === undefined) {
    return { from: start, to: monthEnd(monthIndex(start)), label: start.slice(0, 7) };
  }
  const day = Number(match[3]);
  const lastDay = Number(monthEnd(monthIndex(start)).slice(8, 10));
  if (day < 1 || day > lastDay) return undefined;
  const date = isoDate(year, month, day);
  return { from: date, to: date, label: date };
};
