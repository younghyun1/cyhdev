import { contractApi } from "../account_api";

let countryListRequest: ReturnType<typeof contractApi.getCountries> | null =
  null;

export const healthApi = {
  server: () => contractApi.healthcheck(),
  state: () => contractApi.root(),
  fastfetch: () => contractApi.getHostFastfetch(),
} as const;

export const dropdownApi = {
  languageList: () => contractApi.getLanguages(),
  language: (languageId: string | number) =>
    contractApi.getLanguage({ path: { language_id: Number(languageId) } }),
  countryList: () => {
    if (countryListRequest === null) {
      countryListRequest = contractApi.getCountries().catch((error: unknown) => {
        countryListRequest = null;
        throw error;
      });
    }
    return countryListRequest;
  },
  country: (countryId: string | number) =>
    contractApi.getCountry({ path: { country_id: Number(countryId) } }),
  countrySubdivisions: (countryId: string | number) =>
    contractApi.getSubdivisionsForCountry({
      path: { country_id: Number(countryId) },
    }),
} as const;

export const geoApi = {
  lookupIp: (ipAddress: string) =>
    contractApi.lookupIpLocation({ path: { ip_address: ipAddress } }),
} as const;

export const geoIpApi = {
  getGeoIpInfo: (ipAddress: string) =>
    contractApi.lookupIpInfo({ path: { ip_address: ipAddress } }),
  getMyIpInfo: () => contractApi.lookupMyIpInfo(),
} as const;

export const visitorBoardApi = {
  getVisitorBoard: () => contractApi.getVisitorBoardEntries(),
} as const;
