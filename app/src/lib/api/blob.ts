import { api } from '@/lib/request';

/// Fetch a blob's bytes (authenticated), e.g. to preview an image.
export function fetchBlob(projectId: string, sha256: string): Promise<Blob> {
  return api.get(`project/${projectId}/blobs/${sha256}`).blob();
}

/// Upload raw bytes as a content-addressed blob in the project; returns its
/// sha256 + size so the caller can create a binary file node referencing it.
export async function uploadBlob(
  projectId: string,
  bytes: ArrayBuffer,
): Promise<{ sha256: string; size: number }> {
  const res = await api
    .post(`project/${projectId}/blobs`, { body: bytes })
    .json<{ payload: { sha256: string; size: number } }>();
  return res.payload;
}
