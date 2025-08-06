# Task Grouping Mikado Method Pre-Plan

## Overall Goal

Implement automatic task grouping based on colon-separated namespaces in mise task names, with nested group support and global sequence steps. Both tasks and groups should be navigable, renameable, and assignable to sequence steps through keyboard or mouse interactions.

## Analysis Summary

The current system is tightly coupled to flat task lists with direct indexing (`selected_task: usize`, `tasks[index]`). Implementing hierarchical grouping requires breaking these dependencies systematically to avoid breaking existing functionality.

## Core Changes Required

### 1. **Create NavigableItem System**
- **New file**: `src/models/navigable_item.rs`
- **Purpose**: Unified system for tasks and groups (both can be selected, renamed, assigned to steps)
- **Content**: 
  ```rust
  enum NavigableItem { 
    Task(MiseTask), 
    Group { 
      name: String, 
      children: Vec<NavigableItem>,
      expanded: bool  // for future collapsing support
    } 
  }
  ```
- **Dependencies**: None (leaf change)

### 2. **Replace Task Index Navigation** 
- **Files**: `src/app/mod.rs` 
- **Change**: Replace `selected_task: usize` → `selected_item_path: Vec<String>`
- **Change**: Replace `tasks: Vec<MiseTask>` → `items: Vec<NavigableItem>`
- **Change**: Update all methods that use direct indexing
- **Dependencies**: Requires #1

### 3. **Create Navigation Helper**
- **New file**: `src/models/navigation_helper.rs`
- **Purpose**: Find items by path, get flat navigation list, handle hierarchical navigation
- **Methods**: 
  - `get_item_by_path(path: &[String]) -> Option<&NavigableItem>`
  - `get_flat_navigation_list() -> Vec<(Vec<String>, &NavigableItem)>`
  - `find_next_navigable(current_path: &[String]) -> Option<Vec<String>>`
  - `find_prev_navigable(current_path: &[String]) -> Option<Vec<String>>`
- **Dependencies**: Requires #1, #2

### 4. **Update Task Parsing**
- **File**: `src/mise/client.rs`
- **Change**: Parse colon-separated task names into group hierarchy
- **Change**: Build `NavigableItem` tree instead of flat `MiseTask` list
- **Logic**: `frontend:build:dev` becomes Group("frontend") → Group("build") → Task("dev")
- **Dependencies**: Requires #1

### 5. **Update UI Rendering**
- **File**: `src/ui/sequence_builder.rs` (the big one - 826 lines)
- **Change**: Render groups with indentation levels
- **Change**: Groups show rename controls and can be assigned to sequence steps
- **Change**: Maintain visual hierarchy while keeping sequence step columns aligned
- **Change**: Update button positioning for hierarchical layout
- **Dependencies**: Requires #1, #2, #3

### 6. **Update Event Handlers**
- **File**: `src/app/event_handlers.rs`
- **Change**: Handle navigation with paths instead of indices (`self.selected_item_path`)
- **Change**: Support rename mode for both tasks and groups
- **Change**: Update all keyboard shortcuts to work with path-based navigation
- **Dependencies**: Requires #1, #2, #3

### 7. **Update Mouse Interactions**
- **Files**: `src/app/event_handlers.rs`, `src/ui/button_layout.rs`
- **Change**: Click detection works with hierarchical layout and indentation
- **Change**: Button positioning accounts for group nesting levels
- **Dependencies**: Requires #5, #6

### 8. **Update Sequence Management** 
- **File**: `src/app/sequence_management.rs`
- **Change**: Handle group assignment to sequence steps
- **Change**: When group assigned, include all child tasks in execution
- **Dependencies**: Requires #1, #2, #3

## Dependency Tree

```
1. NavigableItem System (leaf - safe to implement first)
   ↓
2. Replace Task Index Navigation (breaks existing navigation)
   ↓
3. Navigation Helper (provides new navigation logic)
   ↓
   ├── 4. Task Parsing (parallel - changes data source)
   ├── 5. UI Rendering (parallel - needs 1,2,3)
   └── 6. Event Handlers (parallel - needs 1,2,3)
       ↓
       ├── 7. Mouse Interactions (needs 5,6)
       └── 8. Sequence Management (needs 1,2,3)
```

## Implementation Strategy

### Mikado Method Approach:
1. **Small, safe changes first**: Start with #1 (new file, no breaking changes)
2. **One dependency at a time**: Complete each numbered item fully before starting dependents
3. **Maintain working state**: After each change, all existing functionality must work
4. **Test continuously**: Verify keyboard/mouse navigation at every step

### Key Design Decisions:
- **Groups are first-class citizens**: Can be selected, renamed, assigned to sequence steps
- **Path-based navigation**: `["frontend", "build", "dev"]` instead of array indices
- **Flat navigation experience**: Up/down arrows work seamlessly across hierarchy
- **Backward compatibility**: Tasks without colons remain flat in the root level
- **Complete keyboard/mouse support**: Every action works with both input methods

## Expected UI After Implementation

```
┌─ Available Tasks ─────────────────────────────────────────────────────────┐
│ frontend                    │        │        │        │ [rename]          │
│   build                     │        │        │        │ [rename]          │
│   > dev       │   ●    │        │        │ [run] [cat] [edit] [rename] │
│     prod      │        │        │        │ [run] [cat] [edit] [rename] │
│   test                      │        │        │   ●    │ [rename]          │
│     unit      │        │        │   ●    │ [run] [cat] [edit] [rename] │
│ backend                     │        │   ●    │        │ [rename]          │
│   api                       │        │        │        │ [rename]          │
│     start     │        │        │        │ [run] [cat] [edit] [rename] │
│ build         │        │        │        │ [run] [cat] [edit] [rename] │
│ test          │        │   ●    │        │ [run] [cat] [edit] [rename] │
└───────────────────────────────────────────────────────────────────────────┘
```

Groups can be:
- **Selected** with up/down arrows or mouse clicks
- **Renamed** with 'c' key or rename button
- **Assigned to sequence steps** with 1/2/3 keys (assigns all child tasks)
- **Run individually** with 'x' key (runs all child tasks)

This creates an intuitive, hierarchical workflow while maintaining the familiar matrix-style interface.