import React, { useCallback, useMemo, useState } from "react";
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import {
  LexicalTypeaheadMenuPlugin,
  MenuOption,
  useBasicTypeaheadTriggerMatch,
} from "@lexical/react/LexicalTypeaheadMenuPlugin";
import {
  $getSelection,
  $isRangeSelection,
  FORMAT_TEXT_COMMAND,
  LexicalEditor,
} from "lexical";
import { $setBlocksType } from "@lexical/selection";
import { $createHeadingNode, $createQuoteNode } from "@lexical/rich-text";
import {
  INSERT_ORDERED_LIST_COMMAND,
  INSERT_UNORDERED_LIST_COMMAND,
  INSERT_CHECK_LIST_COMMAND,
} from "@lexical/list";
import { createPortal } from "react-dom";

class ComponentPickerOption extends MenuOption {
  title: string;
  icon: string;
  onSelect: (editor: LexicalEditor) => void;

  constructor(
    title: string,
    icon: string,
    onSelect: (editor: LexicalEditor) => void,
  ) {
    super(title);
    this.title = title;
    this.icon = icon;
    this.onSelect = onSelect;
  }
}

export function ComponentPickerPlugin() {
  const [editor] = useLexicalComposerContext();
  const [queryString, setQueryString] = useState<string | null>(null);

  const checkForTriggerMatch = useBasicTypeaheadTriggerMatch("/", {
    minLength: 0,
  });

  const generateOptions = useCallback(() => {
    return [
      new ComponentPickerOption("Heading 1", "format_h1", (editor) => {
        editor.update(() => {
          const selection = $getSelection();
          if ($isRangeSelection(selection)) {
            $setBlocksType(selection, () => $createHeadingNode("h1"));
          }
        });
      }),
      new ComponentPickerOption("Heading 2", "format_h2", (editor) => {
        editor.update(() => {
          const selection = $getSelection();
          if ($isRangeSelection(selection)) {
            $setBlocksType(selection, () => $createHeadingNode("h2"));
          }
        });
      }),
      new ComponentPickerOption("Heading 3", "format_h3", (editor) => {
        editor.update(() => {
          const selection = $getSelection();
          if ($isRangeSelection(selection)) {
            $setBlocksType(selection, () => $createHeadingNode("h3"));
          }
        });
      }),
      new ComponentPickerOption(
        "Bullet List",
        "format_list_bulleted",
        (editor) => {
          editor.dispatchCommand(INSERT_UNORDERED_LIST_COMMAND, undefined);
        },
      ),
      new ComponentPickerOption(
        "Numbered List",
        "format_list_numbered",
        (editor) => {
          editor.dispatchCommand(INSERT_ORDERED_LIST_COMMAND, undefined);
        },
      ),
      new ComponentPickerOption("Check List", "check_box", (editor) => {
        editor.dispatchCommand(INSERT_CHECK_LIST_COMMAND, undefined);
      }),
      new ComponentPickerOption("Quote", "format_quote", (editor) => {
        editor.update(() => {
          const selection = $getSelection();
          if ($isRangeSelection(selection)) {
            $setBlocksType(selection, () => $createQuoteNode());
          }
        });
      }),
    ];
  }, []);

  const options = useMemo(() => {
    const defaultOptions = generateOptions();
    if (!queryString) return defaultOptions;

    return defaultOptions.filter((option) =>
      option.title.toLowerCase().includes(queryString.toLowerCase()),
    );
  }, [queryString, generateOptions]);

  const onSelectOption = useCallback(
    (
      selectedOption: ComponentPickerOption,
      nodeToRemove: any,
      closeMenu: () => void,
      matchingString: string,
    ) => {
      editor.update(() => {
        if (nodeToRemove) {
          nodeToRemove.remove();
        }
        selectedOption.onSelect(editor);
        closeMenu();
      });
    },
    [editor],
  );

  return (
    <LexicalTypeaheadMenuPlugin<ComponentPickerOption>
      onQueryChange={setQueryString}
      onSelectOption={onSelectOption}
      triggerFn={checkForTriggerMatch}
      options={options}
      menuRenderFn={(
        anchorElementRef,
        { selectedIndex, selectOptionAndCleanUp, setHighlightedIndex },
      ) => {
        if (!anchorElementRef.current || options.length === 0) return null;

        return createPortal(
          <div className="bg-white border border-border-main rounded shadow-lg overflow-hidden w-64 absolute z-50 flex flex-col">
            <div className="px-3 py-2 text-xs font-bold text-gray-500 uppercase tracking-widest border-b border-border-main bg-stone-50 shrink-0">
              Basic Blocks
            </div>
            <ul className="max-h-64 overflow-y-auto">
              {options.map((option, i) => (
                <li
                  key={option.key}
                  tabIndex={-1}
                  className={`flex items-center gap-3 px-3 py-2 cursor-pointer text-sm font-medium transition-colors ${
                    selectedIndex === i
                      ? "bg-gray-100/80 text-black border-l-2 border-l-black"
                      : "text-gray-700 hover:bg-stone-50 hover:text-black border-l-2 border-l-transparent"
                  }`}
                  ref={(element) => {
                    if (element && selectedIndex === i) {
                      element.scrollIntoView({ block: "nearest" });
                    }
                  }}
                  onMouseEnter={() => setHighlightedIndex(i)}
                  onClick={() => {
                    setHighlightedIndex(i);
                    selectOptionAndCleanUp(option);
                  }}
                >
                  <span className="material-symbols-outlined icon-sm text-gray-500">
                    {option.icon}
                  </span>
                  {option.title}
                </li>
              ))}
            </ul>
          </div>,
          anchorElementRef.current,
        );
      }}
    />
  );
}
