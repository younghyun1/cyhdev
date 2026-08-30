/** A visitor board entry: [[lat_bytes, lon_bytes], count] */
export type VisitorBoardEntry = [[number, number], number];

export interface VisitorBoardResponse {
  data: VisitorBoardEntry[];
}
