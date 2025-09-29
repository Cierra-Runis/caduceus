import { editor } from 'monaco-editor';

export enum MessageType {
  CURSOR_SELECTION_CHANGED = 'cursorSelectionChanged',
  EDIT = 'edit',
}

export type EditorCursorSelectionChangedMessage = Message<
  MessageType.CURSOR_SELECTION_CHANGED,
  editor.ICursorSelectionChangedEvent
>;

export type EditorEditMessage = Message<
  MessageType.EDIT,
  editor.IModelContentChangedEvent
>;

interface Message<T extends MessageType, D> {
  data: D;
  type: T;
}
