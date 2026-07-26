import { env } from '@/lib/env';
import { api } from '@/lib/request';

/// Fetch a blob's bytes (authenticated), e.g. to preview an image.
export function fetchBlob(projectId: string, sha256: string): Promise<Blob> {
  return api.get(`project/${projectId}/blobs/${sha256}`).blob();
}

/// Upload a file's bytes as a content-addressed blob, reporting progress (0..1).
/// Uses XHR (not ky) because only XHR exposes upload progress. Sends the raw
/// bytes (the server reads the body directly) with the auth cookie.
export function uploadBlobWithProgress(
  projectId: string,
  file: File,
  onProgress: (fraction: number) => void,
): Promise<{ sha256: string; size: number }> {
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    xhr.open('POST', `${env.NEXT_PUBLIC_API_URL}/project/${projectId}/blobs`);
    xhr.withCredentials = true;
    xhr.responseType = 'json';
    xhr.upload.onprogress = (event) => {
      if (event.lengthComputable) onProgress(event.loaded / event.total);
    };
    xhr.onload = () => {
      const payload = xhr.response?.payload;
      if (xhr.status >= 200 && xhr.status < 300 && payload) {
        onProgress(1);
        resolve({ sha256: payload.sha256, size: payload.size });
      } else {
        reject(
          new Error(xhr.response?.message ?? `Upload failed (${xhr.status})`),
        );
      }
    };
    xhr.onerror = () => reject(new Error('Network error during upload'));
    xhr.send(file);
  });
}
