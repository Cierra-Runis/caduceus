import { describe, expect, it, vi } from 'vitest';

const typstMock = vi.hoisted(() => ({
  addSource: vi.fn().mockResolvedValue(undefined),
  setCompilerInitOptions: vi.fn(),
  setRendererInitOptions: vi.fn(),
  svg: vi.fn().mockResolvedValue('<svg></svg>'),
}));

vi.mock('@myriaddreamin/typst.ts', () => ({ $typst: typstMock }));

import { abs, compileProject } from '@/lib/typst';

describe('abs', () => {
  it('prefixes a bare relative path with a single leading slash', () => {
    expect(abs('main.typ')).toBe('/main.typ');
  });

  it('prefixes a nested relative path', () => {
    expect(abs('chapters/intro.typ')).toBe('/chapters/intro.typ');
  });

  it('leaves an already-absolute path untouched', () => {
    expect(abs('/main.typ')).toBe('/main.typ');
  });

  it('collapses repeated leading slashes to one', () => {
    expect(abs('///nested/intro.typ')).toBe('/nested/intro.typ');
  });
});

describe('compileProject', () => {
  it('adds every source at its normalised path and compiles the entry', async () => {
    const svg = await compileProject('main.typ', [
      { path: 'main.typ', text: '= Hello' },
      { path: 'chapters/intro.typ', text: '= Intro' },
    ]);

    expect(typstMock.addSource).toHaveBeenNthCalledWith(1, '/main.typ', '= Hello');
    expect(typstMock.addSource).toHaveBeenNthCalledWith(
      2,
      '/chapters/intro.typ',
      '= Intro',
    );
    expect(typstMock.svg).toHaveBeenCalledWith({ mainFilePath: '/main.typ' });
    expect(svg).toBe('<svg></svg>');
  });

  it('initialises the compiler and renderer modules', async () => {
    await compileProject('main.typ', [{ path: 'main.typ', text: '= Hello' }]);

    expect(typstMock.setCompilerInitOptions).toHaveBeenCalled();
    expect(typstMock.setRendererInitOptions).toHaveBeenCalled();
  });
});
