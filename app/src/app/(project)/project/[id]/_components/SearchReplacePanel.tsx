'use client';

import { ChevronDownIcon, ChevronRightIcon } from 'lucide-react';
import { useTranslations } from 'next-intl';
import { useEffect, useMemo, useState } from 'react';
import * as Y from 'yjs';

import { cn } from '@/lib/utils';

export interface SearchReplacePanelProps {
  onOpen: (path: string) => void;
  /// Text-file paths to search across.
  paths: string[];
  ydoc: Y.Doc;
}

interface FileMatches {
  matches: Match[];
  path: string;
}

interface Match {
  /// 0-based column of the match within its line.
  column: number;
  /// 0-based line index.
  line: number;
  lineText: string;
  matchLength: number;
  /// Absolute offset in the file text.
  start: number;
}

interface Options {
  regex: boolean;
  sensitive: boolean;
  word: boolean;
}

// VS Code-style search & replace across the project's text files. Reads live
// content from the shared Yjs doc, so results reflect unsaved edits and replaces
// flow back through the CRDT (syncing to peers and persisting).
export function SearchReplacePanel({
  onOpen,
  paths,
  ydoc,
}: SearchReplacePanelProps) {
  const t = useTranslations('Search');

  const [query, setQuery] = useState('');
  const [replacement, setReplacement] = useState('');
  const [showReplace, setShowReplace] = useState(false);
  const [options, setOptions] = useState<Options>({
    regex: false,
    sensitive: false,
    word: false,
  });

  // Live content mirror: refreshed on every doc update so results track edits
  // (including replaces applied below).
  const [contents, setContents] = useState<Record<string, string>>({});
  useEffect(() => {
    const sync = () => {
      const next: Record<string, string> = {};
      for (const path of paths) next[path] = ydoc.getText(path).toString();
      setContents(next);
    };
    sync();
    ydoc.on('update', sync);
    return () => ydoc.off('update', sync);
  }, [ydoc, paths]);

  const { error, results, total } = useMemo(
    () => search(query, contents, options),
    [query, contents, options],
  );

  const applyReplace = (path: string, single?: Match) => {
    const regex = buildRegex(query, options, !single);
    if (!regex) return;
    const yText = ydoc.getText(path);
    const text = yText.toString();
    ydoc.transact(() => {
      if (single) {
        const matched = text.slice(single.start, single.start + single.matchLength);
        const value = options.regex
          ? matched.replace(regex, replacement)
          : replacement;
        yText.delete(single.start, single.matchLength);
        yText.insert(single.start, value);
      } else {
        // In regex mode honor `$1`/`$&`; in literal mode treat `$` literally by
        // using a function replacer (avoids accidental substitution).
        const next = options.regex
          ? text.replace(regex, replacement)
          : text.replace(regex, () => replacement);
        if (next !== text) {
          yText.delete(0, yText.length);
          yText.insert(0, next);
        }
      }
    });
  };

  const replaceAll = () => {
    for (const file of results) applyReplace(file.path);
  };

  return (
    <div className='flex h-full flex-col'>
      <div className='flex items-start gap-1 border-b p-2'>
        <button
          aria-label={t('toggleReplace')}
          className='mt-1 rounded-sm p-0.5 hover:bg-accent'
          onClick={() => setShowReplace((v) => !v)}
          type='button'
        >
          {showReplace ? (
            <ChevronDownIcon className='size-4 opacity-70' />
          ) : (
            <ChevronRightIcon className='size-4 opacity-70' />
          )}
        </button>
        <div className='flex flex-1 flex-col gap-1'>
          <div
            className={cn(
              `flex items-center gap-1 rounded-sm border bg-background px-1`,
              error && 'border-destructive',
            )}
          >
            <input
              className='min-w-0 flex-1 bg-transparent py-1 text-sm outline-none'
              onChange={(e) => setQuery(e.target.value)}
              placeholder={t('searchPlaceholder')}
              value={query}
            />
            <Toggle
              active={options.sensitive}
              label={t('matchCase')}
              onClick={() =>
                setOptions((o) => ({ ...o, sensitive: !o.sensitive }))
              }
            >
              Aa
            </Toggle>
            <Toggle
              active={options.word}
              label={t('matchWholeWord')}
              onClick={() => setOptions((o) => ({ ...o, word: !o.word }))}
            >
              <span className='underline'>ab</span>
            </Toggle>
            <Toggle
              active={options.regex}
              label={t('useRegex')}
              onClick={() => setOptions((o) => ({ ...o, regex: !o.regex }))}
            >
              .*
            </Toggle>
          </div>
          {showReplace && (
            <div
              className={`
                flex items-center gap-1 rounded-sm border bg-background px-1
              `}
            >
              <input
                className='min-w-0 flex-1 bg-transparent py-1 text-sm outline-none'
                onChange={(e) => setReplacement(e.target.value)}
                placeholder={t('replacePlaceholder')}
                value={replacement}
              />
              <button
                className={`
                  rounded-sm px-1.5 py-0.5 text-xs hover:bg-accent
                  disabled:opacity-40
                `}
                disabled={total === 0}
                onClick={replaceAll}
                type='button'
              >
                {t('replaceAll')}
              </button>
            </div>
          )}
        </div>
      </div>

      <div className='min-h-0 flex-1 overflow-auto py-1 text-sm'>
        {error ? (
          <p className='px-3 py-1 text-xs text-destructive'>{t('invalidRegex')}</p>
        ) : query && total === 0 ? (
          <p className='px-3 py-1 text-xs opacity-60'>{t('noResults')}</p>
        ) : (
          <>
            {total > 0 && (
              <p className='px-3 py-1 text-xs opacity-60'>
                {t('summary', { files: results.length, matches: total })}
              </p>
            )}
            {results.map((file) => (
              <FileResult
                file={file}
                key={file.path}
                onOpen={onOpen}
                onReplaceMatch={
                  showReplace
                    ? (match) => applyReplace(file.path, match)
                    : undefined
                }
              />
            ))}
          </>
        )}
      </div>
    </div>
  );
}

function buildRegex(
  query: string,
  options: Options,
  global: boolean,
): null | RegExp {
  if (!query) return null;
  let pattern = options.regex ? query : escapeRegExp(query);
  if (options.word) pattern = `\\b${pattern}\\b`;
  const flags = (global ? 'g' : '') + (options.sensitive ? '' : 'i');
  try {
    return new RegExp(pattern, flags);
  } catch {
    return null;
  }
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function FileResult({
  file,
  onOpen,
  onReplaceMatch,
}: {
  file: FileMatches;
  onOpen: (path: string) => void;
  onReplaceMatch?: (match: Match) => void;
}) {
  const [open, setOpen] = useState(true);
  const name = file.path.split('/').pop() ?? file.path;
  const dir = file.path.slice(0, file.path.length - name.length - 1);

  return (
    <div>
      <button
        className='flex w-full items-center gap-1 px-2 py-1 hover:bg-accent/50'
        onClick={() => setOpen((v) => !v)}
        type='button'
      >
        {open ? (
          <ChevronDownIcon className='size-3.5 shrink-0 opacity-60' />
        ) : (
          <ChevronRightIcon className='size-3.5 shrink-0 opacity-60' />
        )}
        <span className='truncate font-medium'>{name}</span>
        {dir && <span className='truncate text-xs opacity-50'>{dir}</span>}
        <span
          className={`
            ml-auto shrink-0 rounded-full bg-muted px-1.5 text-xs opacity-70
          `}
        >
          {file.matches.length}
        </span>
      </button>
      {open &&
        file.matches.map((match, index) => (
          <div
            className='group flex items-center gap-1 pr-2'
            key={`${match.start}-${index}`}
          >
            <button
              className={`
                flex min-w-0 flex-1 items-center py-0.5 pl-7 text-left text-xs
                hover:bg-accent/50
              `}
              onClick={() => onOpen(file.path)}
              type='button'
            >
              <span className='truncate opacity-80'>
                {match.lineText.slice(0, match.column)}
                <mark className='rounded-sm bg-yellow-500/40 text-inherit'>
                  {match.lineText.slice(
                    match.column,
                    match.column + match.matchLength,
                  )}
                </mark>
                {match.lineText.slice(match.column + match.matchLength)}
              </span>
            </button>
            {onReplaceMatch && (
              <button
                className={`
                  shrink-0 rounded-sm px-1 text-xs opacity-0 group-hover:opacity-100
                  hover:bg-accent
                `}
                onClick={() => onReplaceMatch(match)}
                type='button'
              >
                ⇄
              </button>
            )}
          </div>
        ))}
    </div>
  );
}

function lineIndexOf(lineStarts: number[], offset: number): number {
  // Binary search for the last line start <= offset.
  let low = 0;
  let high = lineStarts.length - 1;
  while (low < high) {
    const mid = Math.ceil((low + high) / 2);
    if (lineStarts[mid] <= offset) low = mid;
    else high = mid - 1;
  }
  return low;
}

function lineStartOffsets(text: string): number[] {
  const starts = [0];
  for (let i = 0; i < text.length; i++) {
    if (text[i] === '\n') starts.push(i + 1);
  }
  return starts;
}

function search(
  query: string,
  contents: Record<string, string>,
  options: Options,
): { error: boolean; results: FileMatches[]; total: number } {
  if (!query) return { error: false, results: [], total: 0 };
  const regex = buildRegex(query, options, true);
  if (!regex) return { error: true, results: [], total: 0 };

  const results: FileMatches[] = [];
  let total = 0;
  for (const path of Object.keys(contents).sort()) {
    const text = contents[path];
    const matches: Match[] = [];
    // Precompute line starts to map an offset to line/column.
    const lineStarts = lineStartOffsets(text);
    for (const m of text.matchAll(regex)) {
      const start = m.index ?? 0;
      const matchLength = m[0].length;
      if (matchLength === 0) continue; // avoid infinite empty matches
      const line = lineIndexOf(lineStarts, start);
      const column = start - lineStarts[line];
      const lineEnd = text.indexOf('\n', start);
      const lineText = text.slice(
        lineStarts[line],
        lineEnd === -1 ? text.length : lineEnd,
      );
      matches.push({ column, line, lineText, matchLength, start });
    }
    if (matches.length > 0) {
      results.push({ matches, path });
      total += matches.length;
    }
  }
  return { error: false, results, total };
}

function Toggle({
  active,
  children,
  label,
  onClick,
}: {
  active: boolean;
  children: React.ReactNode;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      aria-label={label}
      className={cn(
        `flex size-6 items-center justify-center rounded-sm text-xs
        hover:bg-accent`,
        active && 'bg-accent text-accent-foreground ring-1 ring-ring',
      )}
      onClick={onClick}
      title={label}
      type='button'
    >
      {children}
    </button>
  );
}
