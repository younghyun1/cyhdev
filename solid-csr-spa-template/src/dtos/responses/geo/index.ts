// Geo IP response DTOs

export interface IpInfo {
  ip: string;
  country_code: string;
  country_name: string;
  state: string;
  city: string;
  postal: string;
  latitude: number;
  longitude: number;
}

/** Response from GET /api/geo-ip-info/{ip_address} */
export type GetGeoIpInfoResponse = IpInfo;

/** Response from GET /api/geo-ip-info/me */
export type GetMyIpInfoResponse = IpInfo;

/** Response from GET /api/geolocate/{ip_address} */
export type GeolocateResponse = IpInfo;
