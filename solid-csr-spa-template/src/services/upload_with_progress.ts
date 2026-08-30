import { ApiContractError, type ApiResponse } from "../generated";
import { apiUrl, handleUnauthorizedResponse } from "./api";

type UploadRequest = {
  readonly url: string;
  readonly formData: FormData;
  readonly onProgress: (percentage: number) => void;
  readonly headers?: Readonly<Record<string, string>>;
  readonly credentials?: RequestCredentials;
};

/** Sends a generated multipart contract while preserving browser upload progress. */
export function uploadWithProgress<T>({
  url,
  formData,
  onProgress,
  headers = {},
  credentials = "include",
}: UploadRequest): Promise<ApiResponse<T>> {
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    xhr.open("POST", apiUrl(url), true);
    xhr.withCredentials = credentials === "include";

    for (const [name, value] of Object.entries(headers)) {
      if (name.toLowerCase() !== "content-type") {
        xhr.setRequestHeader(name, value);
      }
    }

    xhr.upload.onprogress = (event) => {
      if (event.lengthComputable) {
        onProgress(Math.round((event.loaded / event.total) * 100));
      }
    };

    xhr.onload = () => {
      if (xhr.status === 401 || xhr.status === 403) {
        handleUnauthorizedResponse();
      }
      if (xhr.status < 200 || xhr.status >= 300) {
        reject(new ApiContractError(xhr.status, xhr.responseText));
        return;
      }
      try {
        resolve(JSON.parse(xhr.responseText) as ApiResponse<T>);
      } catch (error: unknown) {
        reject(
          error instanceof Error
            ? error
            : new Error("Upload response was not valid JSON"),
        );
      }
    };
    xhr.onerror = () =>
      reject(new ApiContractError(0, "Upload failed: network error"));
    xhr.send(formData);
  });
}
