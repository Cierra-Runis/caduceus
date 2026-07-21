// GENERATED FILE — DO NOT EDIT.
// Emitted by openapi-zod-client from docs/openapi.json (which is itself
// generated from the Rust server; see server/src/openapi.rs).
// Regenerate with `pnpm codegen:api`.
/* eslint-disable */
import { z } from "zod";

const HealthStatus = z.literal("healthy");
const ApiSuccess_HealthPayload = z
  .object({
    message: z.string(),
    payload: z.object({ status: HealthStatus }).passthrough(),
  })
  .passthrough();
const LoginRequest = z
  .object({ password: z.string(), username: z.string() })
  .passthrough();
const UserPayload = z
  .object({
    avatar_uri: z.union([z.string(), z.null()]),
    created_at: z.string().datetime({ offset: true }),
    id: z.string(),
    nickname: z.string(),
    updated_at: z.string().datetime({ offset: true }),
    username: z.string(),
  })
  .passthrough();
const ApiSuccess_AuthPayload = z
  .object({
    message: z.string(),
    payload: z.object({ token: z.string(), user: UserPayload }).passthrough(),
  })
  .passthrough();
const ApiMessage = z.object({ message: z.string() }).passthrough();
const OwnerType = z.enum(["user", "team"]);
const CreateProjectRequest = z
  .object({ name: z.string(), owner_id: z.string(), owner_type: OwnerType })
  .passthrough();
const FileKind = z.enum(["text", "binary"]);
const ProjectFilePayload = z
  .object({
    id: z.string(),
    kind: FileKind,
    path: z.string(),
    size: z.number().int(),
    updated_at: z.string().datetime({ offset: true }),
    version: z.number().int(),
  })
  .passthrough();
const ApiSuccess_ProjectPayload = z
  .object({
    message: z.string(),
    payload: z
      .object({
        created_at: z.string().datetime({ offset: true }),
        creator_id: z.string(),
        entry: z.union([z.string(), z.null()]),
        files: z.array(ProjectFilePayload),
        id: z.string(),
        name: z.string(),
        owner_id: z.string(),
        owner_type: OwnerType,
        pinned_version: z.union([z.string(), z.null()]),
        updated_at: z.string().datetime({ offset: true }),
      })
      .passthrough(),
  })
  .passthrough();
const FileContentPayload = z.union([
  z.object({ kind: z.literal("text"), text: z.string() }).passthrough(),
  z.object({ kind: z.literal("binary"), storageKey: z.string() }).passthrough(),
]);
const ProjectFileDetailPayload = z
  .object({
    content: FileContentPayload,
    id: z.string(),
    path: z.string(),
    size: z.number().int(),
    updated_at: z.string().datetime({ offset: true }),
    version: z.number().int(),
  })
  .passthrough();
const ApiSuccess_ProjectDetailPayload = z
  .object({
    message: z.string(),
    payload: z
      .object({
        created_at: z.string().datetime({ offset: true }),
        creator_id: z.string(),
        entry: z.union([z.string(), z.null()]),
        files: z.array(ProjectFileDetailPayload),
        id: z.string(),
        name: z.string(),
        owner_id: z.string(),
        owner_type: OwnerType,
        pinned_version: z.union([z.string(), z.null()]),
        updated_at: z.string().datetime({ offset: true }),
      })
      .passthrough(),
  })
  .passthrough();
const UpdateFileRequest = z.object({ text: z.string() }).passthrough();
const ApiSuccess_UpdateFilePayload = z
  .object({
    message: z.string(),
    payload: z
      .object({
        id: z.string(),
        updated_at: z.string().datetime({ offset: true }),
        version: z.number().int(),
      })
      .passthrough(),
  })
  .passthrough();
const RegisterRequest = z
  .object({ password: z.string(), username: z.string() })
  .passthrough();
const CreateTeamRequest = z.object({ name: z.string() }).passthrough();
const ApiSuccess_TeamPayload = z
  .object({
    message: z.string(),
    payload: z
      .object({
        avatar_uri: z.union([z.string(), z.null()]),
        created_at: z.string().datetime({ offset: true }),
        creator_id: z.string(),
        id: z.string(),
        member_ids: z.array(z.string()),
        name: z.string(),
        updated_at: z.string().datetime({ offset: true }),
      })
      .passthrough(),
  })
  .passthrough();
const ApiSuccess_Vec_ProjectPayload = z
  .object({
    message: z.string(),
    payload: z.array(
      z
        .object({
          created_at: z.string().datetime({ offset: true }),
          creator_id: z.string(),
          entry: z.union([z.string(), z.null()]),
          files: z.array(ProjectFilePayload),
          id: z.string(),
          name: z.string(),
          owner_id: z.string(),
          owner_type: OwnerType,
          pinned_version: z.union([z.string(), z.null()]),
          updated_at: z.string().datetime({ offset: true }),
        })
        .passthrough()
    ),
  })
  .passthrough();
const ApiSuccess_UserPayload = z
  .object({
    message: z.string(),
    payload: z
      .object({
        avatar_uri: z.union([z.string(), z.null()]),
        created_at: z.string().datetime({ offset: true }),
        id: z.string(),
        nickname: z.string(),
        updated_at: z.string().datetime({ offset: true }),
        username: z.string(),
      })
      .passthrough(),
  })
  .passthrough();
const ApiSuccess_Vec_TeamPayload = z
  .object({
    message: z.string(),
    payload: z.array(
      z
        .object({
          avatar_uri: z.union([z.string(), z.null()]),
          created_at: z.string().datetime({ offset: true }),
          creator_id: z.string(),
          id: z.string(),
          member_ids: z.array(z.string()),
          name: z.string(),
          updated_at: z.string().datetime({ offset: true }),
        })
        .passthrough()
    ),
  })
  .passthrough();
const AuthPayload = z
  .object({ token: z.string(), user: UserPayload })
  .passthrough();
const HealthPayload = z.object({ status: HealthStatus }).passthrough();
const ProjectDetailPayload = z
  .object({
    created_at: z.string().datetime({ offset: true }),
    creator_id: z.string(),
    entry: z.union([z.string(), z.null()]),
    files: z.array(ProjectFileDetailPayload),
    id: z.string(),
    name: z.string(),
    owner_id: z.string(),
    owner_type: OwnerType,
    pinned_version: z.union([z.string(), z.null()]),
    updated_at: z.string().datetime({ offset: true }),
  })
  .passthrough();
const ProjectPayload = z
  .object({
    created_at: z.string().datetime({ offset: true }),
    creator_id: z.string(),
    entry: z.union([z.string(), z.null()]),
    files: z.array(ProjectFilePayload),
    id: z.string(),
    name: z.string(),
    owner_id: z.string(),
    owner_type: OwnerType,
    pinned_version: z.union([z.string(), z.null()]),
    updated_at: z.string().datetime({ offset: true }),
  })
  .passthrough();
const TeamPayload = z
  .object({
    avatar_uri: z.union([z.string(), z.null()]),
    created_at: z.string().datetime({ offset: true }),
    creator_id: z.string(),
    id: z.string(),
    member_ids: z.array(z.string()),
    name: z.string(),
    updated_at: z.string().datetime({ offset: true }),
  })
  .passthrough();
const UpdateFilePayload = z
  .object({
    id: z.string(),
    updated_at: z.string().datetime({ offset: true }),
    version: z.number().int(),
  })
  .passthrough();

export const schemas = {
  HealthStatus,
  ApiSuccess_HealthPayload,
  LoginRequest,
  UserPayload,
  ApiSuccess_AuthPayload,
  ApiMessage,
  OwnerType,
  CreateProjectRequest,
  FileKind,
  ProjectFilePayload,
  ApiSuccess_ProjectPayload,
  FileContentPayload,
  ProjectFileDetailPayload,
  ApiSuccess_ProjectDetailPayload,
  UpdateFileRequest,
  ApiSuccess_UpdateFilePayload,
  RegisterRequest,
  CreateTeamRequest,
  ApiSuccess_TeamPayload,
  ApiSuccess_Vec_ProjectPayload,
  ApiSuccess_UserPayload,
  ApiSuccess_Vec_TeamPayload,
  AuthPayload,
  HealthPayload,
  ProjectDetailPayload,
  ProjectPayload,
  TeamPayload,
  UpdateFilePayload,
};
