// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type {
  ApiResponse,
  CountryAndSubdivisions,
  GetCountriesResponse,
  IpInfo,
  IsoCountrySubdivision,
  IsoLanguage,
  RootHandlerResponse,
  ServerHealthcheckResponse,
} from "../api-types";
import {
  interpolatePath,
  requestHeaders,
  requestJson,
  type ApiRequestOptions,
  type ApiTransport,
} from "../runtime";

export function createReferenceClient(transport: ApiTransport) {
  return {
    getCountries: async (options: ApiRequestOptions = {}) => {
      const path = "/api/dropdown/country";
      const url = path;
      return requestJson<ApiResponse<GetCountriesResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    getCountry: async (input: {
      readonly path: {
        readonly country_id: number;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/dropdown/country/{country_id}", input.path);
      const url = path;
      return requestJson<ApiResponse<CountryAndSubdivisions>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    getHostFastfetch: async (options: ApiRequestOptions = {}) => {
      const path = "/api/healthcheck/fastfetch";
      const url = path;
      return requestJson<ApiResponse<string>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    getLanguage: async (input: {
      readonly path: {
        readonly language_id: number;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/dropdown/language/{language_id}", input.path);
      const url = path;
      return requestJson<ApiResponse<IsoLanguage>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    getLanguages: async (options: ApiRequestOptions = {}) => {
      const path = "/api/dropdown/language";
      const url = path;
      return requestJson<ApiResponse<ReadonlyArray<IsoLanguage>>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    getSubdivisionsForCountry: async (input: {
      readonly path: {
        readonly country_id: number;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/dropdown/country/{country_id}/subdivision", input.path);
      const url = path;
      return requestJson<ApiResponse<ReadonlyArray<IsoCountrySubdivision>>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    getVisitorBoardEntries: async (options: ApiRequestOptions = {}) => {
      const path = "/api/visitor-board";
      const url = path;
      return requestJson<ApiResponse<ReadonlyArray<readonly [readonly [number, number], number]>>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    healthcheck: async (options: ApiRequestOptions = {}) => {
      const path = "/api/healthcheck/server";
      const url = path;
      return requestJson<ServerHealthcheckResponse>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    lookupIpInfo: async (input: {
      readonly path: {
        readonly ip_address: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/geo-ip-info/{ip_address}", input.path);
      const url = path;
      return requestJson<ApiResponse<IpInfo>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    lookupIpLocation: async (input: {
      readonly path: {
        readonly ip_address: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/geolocate/{ip_address}", input.path);
      const url = path;
      return requestJson<ApiResponse<IpInfo>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    lookupMyIpInfo: async (options: ApiRequestOptions = {}) => {
      const path = "/api/geo-ip-info/me";
      const url = path;
      return requestJson<ApiResponse<IpInfo>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    root: async (options: ApiRequestOptions = {}) => {
      const path = "/api/healthcheck/state";
      const url = path;
      return requestJson<ApiResponse<RootHandlerResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
  } as const;
}
