# Brainstorm: Seamless Search UI

**Status**: Ready for specification
**Captured**: 2026-03-08

## Idea

Merge the Browsing and Filtering modes into a single seamless experience. Instead of requiring `/` to enter search mode, any character typed immediately starts filtering. This eliminates the mode switch friction and makes the TUI feel more like fzf or other modern fuzzy finders.

## Key Changes

### 1. Merge Browsing + Filtering into one mode

Currently: Browsing (navigate) -> press `/` -> Filtering (type query) -> Esc -> Browsing
Proposed: Single mode where typing starts filtering immediately, navigation keys (arrows, pgup/down, home/end) move the cursor, Escape clears filter or quits.

**Implications**:
- `Mode::Browsing` and `Mode::Filtering` merge into a single `Mode::List` (or just remove the distinction)
- The `/` key is no longer special, it types a `/` into the filter
- `q` types into the filter instead of quitting (quit via Esc 2x or Ctrl-C)
- `j`/`k` still navigate (they don't type into filter since they're navigation keys)

**Wait, conflict**: If `j` and `k` navigate, the user can't type "j" or "k" in the filter. We need to decide: should `j`/`k` navigate or type?

**Resolution**: `j`/`k` as vim navigation keys conflict with typing. Options:
- Use only arrow keys for navigation (j/k type into filter)
- Keep j/k as navigation when filter is empty, type when filter has text

The cleanest approach: **j/k type into the filter** like any other character. Navigation is via arrow keys, pgup/pgdown, home/end only. This is how fzf works.

### 2. Status bar with inline filter

When no filter is active:
```
 Enter detail  Esc quit  (type to search)
```

When filter is active:
```
 filter: auth  17/418 matches  (searching content...)  Esc clear  Enter detail
```

The filter text is colored (e.g., cyan) to stand out. Match count shown. Content search spinner integrated.

### 3. Selected line highlighting

Replace the current subtle `Rgb(30,30,50)` background with a more prominent full-width colored bar (like fzf). The cursor arrow `▸` remains in a contrasting color.

### 4. Escape behavior

- First Escape: clears filter text (if any), cancels content search, restores full list
- Second Escape (when filter is already empty): quits the app
- Ctrl-C: always quits immediately

### 5. Conversation viewer status bar

Replace `" SESSION " project_name` with just `project_name (branch)`:
```
 cc-session (main)  Space/b scroll  g/G top/bottom  / search  Enter copy & exit  Esc back
```

No label badge needed, the directory context is self-evident.

## Implementation Notes

- Remove `Mode::Browsing` and `Mode::Filtering` distinction (merge into single mode)
- In `handle_input`, all printable characters go to filter except navigation keys
- Navigation keys: Up/Down arrows, PageUp/PageDown, Home/End, Enter, Escape, Ctrl-C
- All other keys (letters, numbers, symbols, space): type into filter
- Backspace: remove last filter character
- The debounced content search behavior from 005 stays the same
- Selected line bg: use a stronger theme color (add `selected_bg` to Theme if not already there, make it more visible)

## Open Questions

- Should `Tab` do anything? (e.g., cycle through results, or toggle between metadata/content matches)
- Should we show a cursor blinking indicator in the status bar filter text?
- Should Home/End go to first/last item in the list, or should they be typeable?
