import {
  INSERT_ORDERED_LIST_COMMAND,
  INSERT_UNORDERED_LIST_COMMAND,
} from "@lexical/list";
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { $createHeadingNode, $createQuoteNode } from "@lexical/rich-text";
import { $setBlocksType } from "@lexical/selection";
import {
  $getSelection,
  $isRangeSelection,
  FORMAT_ELEMENT_COMMAND,
  FORMAT_TEXT_COMMAND,
  REDO_COMMAND,
  UNDO_COMMAND,
} from "lexical";

export function ToolbarPlugin() {
  const [editor] = useLexicalComposerContext();

  const handleFormat = (format: "bold" | "italic" | "underline" | "code") => {
    editor.dispatchCommand(FORMAT_TEXT_COMMAND, format);
  };

  const align = (direction: "left" | "center" | "right" | "justify") => {
    editor.dispatchCommand(FORMAT_ELEMENT_COMMAND, direction);
  };

  const formatHeading = (headingSize: "h1" | "h2" | "h3") => {
    editor.update(() => {
      const selection = $getSelection();
      if ($isRangeSelection(selection)) {
        $setBlocksType(selection, () => $createHeadingNode(headingSize));
      }
    });
  };

  const formatQuote = () => {
    editor.update(() => {
      const selection = $getSelection();
      if ($isRangeSelection(selection)) {
        $setBlocksType(selection, () => $createQuoteNode());
      }
    });
  };

  return (
    <div className="flex flex-wrap items-center gap-1 p-2 bg-bg-main border-b border-border-bold sticky top-0 z-10 shrink-0">
      <button
        type="button"
        onClick={() => editor.dispatchCommand(UNDO_COMMAND, undefined)}
        className="p-1.5 hover:bg-bg-sources text-gray-500 rounded-sm hover:text-primary transition-colors"
        title="Undo"
      >
        <span className="material-symbols-outlined icon-sm">undo</span>
      </button>
      <button
        type="button"
        onClick={() => editor.dispatchCommand(REDO_COMMAND, undefined)}
        className="p-1.5 hover:bg-bg-sources text-gray-500 rounded-sm mr-2 hover:text-primary transition-colors"
        title="Redo"
      >
        <span className="material-symbols-outlined icon-sm">redo</span>
      </button>
      <div className="w-px h-4 bg-border-light mx-1" />

      <button
        type="button"
        onClick={() => formatHeading("h1")}
        className="px-2 py-1 text-xs font-bold hover:bg-bg-sources text-gray-700 hover:text-primary rounded-sm transition-colors"
        title="Heading 1"
      >
        H1
      </button>
      <button
        type="button"
        onClick={() => formatHeading("h2")}
        className="px-2 py-1 text-xs font-bold hover:bg-bg-sources text-gray-700 hover:text-primary rounded-sm transition-colors"
        title="Heading 2"
      >
        H2
      </button>
      <button
        type="button"
        onClick={() => formatHeading("h3")}
        className="px-2 py-1 text-xs font-bold hover:bg-bg-sources text-gray-700 hover:text-primary rounded-sm transition-colors"
        title="Heading 3"
      >
        H3
      </button>

      <div className="w-px h-4 bg-border-light mx-1" />

      <button
        type="button"
        onClick={() => handleFormat("bold")}
        className="p-1.5 hover:bg-bg-sources text-gray-700 hover:text-primary rounded-sm font-bold transition-colors"
        title="Bold"
      >
        <span className="material-symbols-outlined icon-sm">format_bold</span>
      </button>
      <button
        type="button"
        onClick={() => handleFormat("italic")}
        className="p-1.5 hover:bg-bg-sources text-gray-700 hover:text-primary rounded-sm italic transition-colors"
        title="Italic"
      >
        <span className="material-symbols-outlined icon-sm">format_italic</span>
      </button>
      <button
        type="button"
        onClick={() => handleFormat("underline")}
        className="p-1.5 hover:bg-bg-sources text-gray-700 hover:text-primary rounded-sm underline transition-colors"
        title="Underline"
      >
        <span className="material-symbols-outlined icon-sm">
          format_underlined
        </span>
      </button>
      <button
        type="button"
        onClick={() => handleFormat("code")}
        className="p-1.5 hover:bg-bg-sources text-gray-700 hover:text-primary rounded-sm font-mono text-xs transition-colors"
        title="Code"
      >
        <span className="material-symbols-outlined icon-sm">code</span>
      </button>

      <div className="w-px h-4 bg-border-light mx-1" />

      <button
        type="button"
        onClick={() =>
          editor.dispatchCommand(INSERT_UNORDERED_LIST_COMMAND, undefined)
        }
        className="p-1.5 hover:bg-bg-sources text-gray-700 hover:text-primary rounded-sm transition-colors"
        title="Bullet List"
      >
        <span className="material-symbols-outlined icon-sm">
          format_list_bulleted
        </span>
      </button>
      <button
        type="button"
        onClick={() =>
          editor.dispatchCommand(INSERT_ORDERED_LIST_COMMAND, undefined)
        }
        className="p-1.5 hover:bg-bg-sources text-gray-700 hover:text-primary rounded-sm transition-colors"
        title="Numbered List"
      >
        <span className="material-symbols-outlined icon-sm">
          format_list_numbered
        </span>
      </button>
      <button
        type="button"
        onClick={formatQuote}
        className="p-1.5 hover:bg-bg-sources text-gray-700 hover:text-primary rounded-sm transition-colors"
        title="Quote block"
      >
        <span className="material-symbols-outlined icon-sm">format_quote</span>
      </button>
    </div>
  );
}
