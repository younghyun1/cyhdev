// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { IsoCountry } from "./iso-country";
import type { IsoCountrySubdivision } from "./iso-country-subdivision";

export type CountryAndSubdivisions = {
  readonly country: IsoCountry;
  readonly subdivisions: ReadonlyArray<IsoCountrySubdivision>;
};
