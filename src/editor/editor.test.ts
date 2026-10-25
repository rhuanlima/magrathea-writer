import { describe, it, expect } from 'vitest';
import { EditorState } from '@codemirror/state';
import { markdown } from '@codemirror/lang-markdown';

describe('CodeMirror 6 Editor Foundation', () => {
  it('creates an editor state with initial markdown text', () => {
    const initialText = '# Title\n\nFirst paragraph.';
    const state = EditorState.create({
      doc: initialText,
      extensions: [markdown()],
    });

    expect(state.doc.toString()).toBe(initialText);
    expect(state.doc.lines).toBe(3);
  });

  it('correctly dispatches text insertions into editor state', () => {
    const state = EditorState.create({
      doc: 'Hello',
    });

    const transaction = state.update({
      changes: { from: 5, insert: ' World' },
    });

    expect(transaction.newDoc.toString()).toBe('Hello World');
  });
});
