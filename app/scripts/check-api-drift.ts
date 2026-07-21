/**
 * API contract drift check (`pnpm test:api-contract`).
 *
 * For every REST endpoint a wire-shaped fixture is parsed twice:
 *   1. by the GENERATED schema (from docs/openapi.json) — proves the fixture
 *      matches what the server declares it sends;
 *   2. by the HAND-WRITTEN schema the app actually uses — proves the app
 *      still understands what the server sends.
 * If the server changes a payload, step 1 breaks on regeneration; fixing the
 * fixture then breaks step 2 until the hand-written schema is updated. The
 * hand-written layer keeps its extra behavior (`.trim()`, string -> Date
 * transforms), which step 3 asserts: every *_at field parses to a valid Date.
 *
 * Plain tsx script (not vitest) so it does not depend on test infra that is
 * not merged yet, and so `*.test.ts` globs never pick it up.
 */

// Hand-written schema modules transitively import '@/lib/env', which
// validates env vars at import time — provide dummies before importing.
process.env.NEXT_PUBLIC_API_URL ??= 'http://localhost:8080/api';
process.env.NEXT_PUBLIC_WS_URL ??= 'ws://localhost:8080/ws';
process.env.JWT_SECRET ??= 'drift-check-dummy';
// NODE_ENV is typed readonly by Next's ambient types; runtime assignment is fine
(process.env as Record<string, string | undefined>).NODE_ENV ??= 'test';

import type * as z from 'zod';

const { schemas } = await import('../src/lib/api/generated/schemas');
const { HealthResponseSchema } = await import('../src/lib/api/health');
const { LoginResponseSchema } = await import('../src/lib/api/login');
const { RegisterResponseSchema } = await import('../src/lib/api/register');
const { CreateTeamResponseSchema, TeamProjectResponseSchema } = await import(
  '../src/lib/api/team'
);
const {
  CreateProjectResponseSchema,
  DuplicateProjectResponseSchema,
  ProjectDetailResponseSchema,
  UpdateFileResponseSchema,
} = await import('../src/lib/api/project');
const { RouteUserMeSchema } = await import('../src/lib/api/user/me');
const { RouteUserTeamsSchema } = await import('../src/lib/api/user/team');
const { UserProjectResponseSchema } = await import('../src/lib/api/user/project');

// ---------------------------------------------------------------------------
// Wire-shaped fixtures. Exercise both branches of every nullable field and
// both FileContent variants.
// ---------------------------------------------------------------------------

const oid = (c: string) => c.repeat(24);
const ts = '2026-07-21T08:30:00Z';

const userPayload = {
  avatar_uri: null,
  created_at: ts,
  id: oid('a'),
  nickname: 'Nick',
  updated_at: '2026-07-21T16:30:00+08:00',
  username: 'user-1',
};

const teamPayload = {
  avatar_uri: 'https://example.com/a.png',
  created_at: ts,
  creator_id: oid('a'),
  id: oid('f'),
  member_ids: [oid('a'), oid('b')],
  name: 'Team',
  updated_at: ts,
};

const filePayload = {
  id: oid('b'),
  kind: 'text',
  path: 'main.typ',
  size: 42,
  updated_at: ts,
  version: 3,
};

const projectPayload = {
  created_at: ts,
  creator_id: oid('c'),
  entry: null,
  files: [filePayload],
  id: oid('d'),
  name: 'Thesis',
  owner_id: oid('c'),
  owner_type: 'user',
  pinned_version: null,
  updated_at: ts,
};

const teamProjectPayload = {
  ...projectPayload,
  entry: oid('b'),
  id: oid('e'),
  owner_type: 'team',
  pinned_version: '0.13.1',
};

const projectDetailPayload = {
  ...projectPayload,
  entry: oid('b'),
  files: [
    {
      content: { kind: 'text', text: '= Hi' },
      id: oid('b'),
      path: 'main.typ',
      size: 4,
      updated_at: ts,
      version: 1,
    },
    {
      content: { kind: 'binary', storageKey: oid('9') },
      id: oid('8'),
      path: 'img/logo.png',
      size: 1024,
      updated_at: ts,
      version: 1,
    },
  ],
};

const authBody = { message: 'ok', payload: { token: 'jwt', user: userPayload } };

// ---------------------------------------------------------------------------

interface Case {
  endpoint: string;
  generated: z.ZodType;
  handwritten: z.ZodType;
  sample: unknown;
}

const cases: Case[] = [
  {
    endpoint: 'GET /api/health',
    generated: schemas.ApiSuccess_HealthPayload,
    handwritten: HealthResponseSchema,
    sample: { message: 'ok', payload: { status: 'healthy' } },
  },
  {
    endpoint: 'POST /api/register',
    generated: schemas.ApiSuccess_AuthPayload,
    handwritten: RegisterResponseSchema,
    sample: authBody,
  },
  {
    endpoint: 'POST /api/login',
    generated: schemas.ApiSuccess_AuthPayload,
    handwritten: LoginResponseSchema,
    sample: authBody,
  },
  {
    endpoint: 'POST /api/team',
    generated: schemas.ApiSuccess_TeamPayload,
    handwritten: CreateTeamResponseSchema,
    sample: { message: 'ok', payload: teamPayload },
  },
  {
    endpoint: 'GET /api/team/projects',
    generated: schemas.ApiSuccess_Vec_ProjectPayload,
    handwritten: TeamProjectResponseSchema,
    sample: { message: 'ok', payload: [projectPayload, teamProjectPayload] },
  },
  {
    endpoint: 'POST /api/project',
    generated: schemas.ApiSuccess_ProjectPayload,
    handwritten: CreateProjectResponseSchema,
    sample: { message: 'ok', payload: projectPayload },
  },
  {
    endpoint: 'POST /api/project/{id}/duplicate',
    generated: schemas.ApiSuccess_ProjectPayload,
    handwritten: DuplicateProjectResponseSchema,
    sample: { message: 'ok', payload: teamProjectPayload },
  },
  {
    endpoint: 'GET /api/project/{id}',
    generated: schemas.ApiSuccess_ProjectDetailPayload,
    handwritten: ProjectDetailResponseSchema,
    sample: { message: 'ok', payload: projectDetailPayload },
  },
  {
    endpoint: 'PUT /api/project/{id}/file/{file_id}',
    generated: schemas.ApiSuccess_UpdateFilePayload,
    handwritten: UpdateFileResponseSchema,
    sample: { message: 'ok', payload: { id: oid('b'), updated_at: ts, version: 4 } },
  },
  {
    endpoint: 'GET /api/user/me',
    generated: schemas.ApiSuccess_UserPayload,
    handwritten: RouteUserMeSchema,
    sample: { message: 'ok', payload: { ...userPayload, avatar_uri: 'https://example.com/me.png' } },
  },
  {
    endpoint: 'GET /api/user/teams',
    generated: schemas.ApiSuccess_Vec_TeamPayload,
    handwritten: RouteUserTeamsSchema,
    sample: { message: 'ok', payload: [teamPayload, { ...teamPayload, avatar_uri: null, id: oid('1') }] },
  },
  {
    endpoint: 'GET /api/user/projects',
    generated: schemas.ApiSuccess_Vec_ProjectPayload,
    handwritten: UserProjectResponseSchema,
    sample: { message: 'ok', payload: [teamProjectPayload] },
  },
];

// Every created_at/updated_at surviving the hand-written parse must have been
// transformed into a valid Date (UI calls .toLocaleDateString() on them).
function collectBadDates(value: unknown, path: string, bad: string[]): void {
  if (Array.isArray(value)) {
    value.forEach((v, i) => collectBadDates(v, `${path}[${i}]`, bad));
  } else if (value !== null && typeof value === 'object' && !(value instanceof Date)) {
    for (const [k, v] of Object.entries(value)) {
      if (k === 'created_at' || k === 'updated_at') {
        if (!(v instanceof Date) || Number.isNaN(v.getTime())) {
          bad.push(`${path}.${k} is not a valid Date (got ${JSON.stringify(v)})`);
        }
      } else {
        collectBadDates(v, `${path}.${k}`, bad);
      }
    }
  }
}

let failures = 0;

for (const { endpoint, generated, handwritten, sample } of cases) {
  const gen = generated.safeParse(sample);
  if (!gen.success) {
    failures++;
    console.error(`FAIL ${endpoint}: fixture rejected by GENERATED schema (fixture vs openapi.json mismatch)`);
    console.error(gen.error.message);
    continue;
  }
  const hand = handwritten.safeParse(sample);
  if (!hand.success) {
    failures++;
    console.error(`FAIL ${endpoint}: wire shape rejected by HAND-WRITTEN schema (server/app drift)`);
    console.error(hand.error.message);
    continue;
  }
  const badDates: string[] = [];
  collectBadDates(hand.data, '$', badDates);
  if (badDates.length > 0) {
    failures++;
    console.error(`FAIL ${endpoint}: date transform broken:\n  ${badDates.join('\n  ')}`);
    continue;
  }
  console.log(`ok   ${endpoint}`);
}

if (failures > 0) {
  console.error(`\n${failures} endpoint(s) drifted between server spec and app schemas`);
  process.exit(1);
}
console.log(`\nall ${cases.length} endpoints in sync`);
