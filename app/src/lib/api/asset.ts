import * as z from 'zod';

import { api } from '@/lib/request';
import { ProjectDetail, ProjectFile, ProjectFileSchema } from '@/lib/types/project';
import { TypstBinaryFile } from '@/lib/typst';

// Response for uploading a binary asset (POST /project/{id}/asset): the newly
// created file, a `binary` reference carrying its storage key.
export type UploadAssetResponse = z.infer<typeof UploadAssetResponseSchema>;
export const UploadAssetResponseSchema = z.object({
  message: z.string().trim(),
  payload: ProjectFileSchema,
});

// Fetch a single asset's raw bytes (GET /project/{id}/asset/{fileId}).
export async function fetchAssetBytes(
  projectId: string,
  fileId: string,
): Promise<Uint8Array> {
  const buffer = await api.get(`project/${projectId}/asset/${fileId}`).arrayBuffer();
  return new Uint8Array(buffer);
}

// Fetch every binary asset of a project, ready to map into the Typst compiler
// VFS. Text files are skipped — their content is already inlined in the detail
// payload. Fetches run concurrently.
export async function fetchBinaryAssets(
  project: ProjectDetail,
): Promise<TypstBinaryFile[]> {
  const binaries = project.files.filter((file) => file.content.kind === 'binary');
  return Promise.all(
    binaries.map(async (file) => ({
      bytes: await fetchAssetBytes(project.id, file.id),
      path: file.path,
    })),
  );
}

// Upload a file as a project asset. `path` is the virtual-FS path the asset
// takes within the project — the same name a Typst `#image("…")` resolves — and
// defaults to the uploaded file's name. The raw bytes are the request body; the
// browser-supplied MIME type is preserved so the asset serves back correctly.
export async function uploadAsset(
  projectId: string,
  path: string,
  file: File,
): Promise<ProjectFile> {
  const res = await api
    .post(`project/${projectId}/asset`, {
      body: file,
      headers: { 'content-type': file.type || 'application/octet-stream' },
      searchParams: { path },
    })
    .json();
  return UploadAssetResponseSchema.parse(res).payload;
}
