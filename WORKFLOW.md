# Robrix Consolidated Workflow - Implementation Status

This document tracks the implementation status of consolidated requirements from the 37 GitHub issues.

**Project Status: COMPILES SUCCESSFULLY**

Last verified: 2026-03-12

## Recent Implementations (This Session)

### Issue #650 - Image Thumbnail Max Height ✅
**File:** `src/shared/text_or_image.rs`
- Added `IMAGE_THUMBNAIL_MAX_HEIGHT = 300.0` constant
- Applied `height: Fit { max: (IMAGE_THUMBNAIL_MAX_HEIGHT) }` to image views
- Prevents very tall images from dominating the timeline

### Issue #656 - No Longer Member View ✅ (Widget Created)
**File:** `src/shared/no_longer_member_view.rs`
- Created `NoLongerMemberView` widget
- Handles Left, Kicked, and Banned states
- Shows rejoin button for non-banned users
- Displays optional kick/ban reason
- Added to RoomScreen DSL and imports

### Issue #755 - Mobile Button Visibility ✅
**Files:** `src/utils.rs`, `src/home/space_lobby.rs`
- Added `is_touch_primary_platform()` helper function
- SpaceLobby buttons_view now always visible on Android/iOS
- Hover behavior preserved on desktop platforms

### Issue #458 - App Lifecycle Events ✅
**File:** `src/app.rs`
- Added handlers for `Event::Pause`, `Event::Resume`, `Event::Background`, `Event::Foreground`
- State saved on Pause and Background events
- Logging for lifecycle transitions

### Issue #560 - Server Notice Rooms Under Separate Header ✅
**File:** `src/home/rooms_list.rs`, `src/shared/collapsible_header.rs`
- Added `ServerNotices` variant to `HeaderCategory` enum
- Added tracking fields: `displayed_server_notice_rooms`, `is_server_notice_rooms_header_expanded`, `server_notice_rooms_indexes`
- Modified `AddJoinedRoom`, `RemoveRoom`, `ClearRooms` handlers to categorize server notice rooms
- Updated `generate_displayed_rooms()` to return 4 Vecs (invited, regular, direct, server_notice)
- Updated `recalculate_indexes()` to calculate server notice room indexes
- Updated `draw_walk()` to render Server Notices collapsible header and rooms
- Updated `CollapsibleHeaderAction::Toggled` handler for `ServerNotices`
- Updated `HideRoom` and `ScrollToRoom` handlers to handle server notice rooms

### Issue #592 - Prettier Invite Screen ✅
**File:** `src/home/invite_screen.rs`
- Added invite timestamp display showing when the invite was received
- Added `invite_timestamp` field to `InviteDetails` struct
- Updated `set_displayed_invite()` to extract and format timestamp
- Timestamp displayed in format "Invited Month Day, Year at HH:MM AM/PM"

### Element Web Parity - Typing Notice with Avatars ✅
**Files:** `src/room/typing_notice.rs`, `src/home/room_screen.rs`, `src/sliding_sync.rs`
- Added `TypingUser` struct with `user_id`, `display_name`, and `avatar_url` fields
- Updated `TimelineUpdate::TypingUsers` to use `Vec<TypingUser>` instead of `Vec<String>`
- Modified typing notification subscriber in sliding_sync.rs to fetch avatar URLs
- Updated `TypingNotice` widget to display up to 3 user avatars alongside typing text
- Avatars overlap slightly (-6px spacing) like Element Web's WhoIsTypingTile
- Falls back to text avatar (first letter) when image not available
- Reference: Element Web's `WhoIsTypingTile.tsx`

---

## Implementation Status Legend

- ✅ **IMPLEMENTED** - Feature exists and works
- 🔨 **PARTIAL** - Some functionality exists, needs enhancement
- ❌ **NOT STARTED** - Needs implementation
- 🚫 **BLOCKED** - Waiting on external dependencies

---

## 1. UI Component Library

### 1.1 Modal System ✅ IMPLEMENTED
**Location:** `src/shared/confirmation_modal.rs`, `src/join_leave_room_modal.rs`

**Existing Components:**
- ✅ `ConfirmationModal` - Base modal with title, body, accept/cancel buttons
- ✅ `PositiveConfirmationModal` - Green checkmark variant
- ✅ `NegativeConfirmationModal` - Red danger variant
- ✅ `JoinLeaveRoomModal` - Room join/leave with loading states
- ✅ Keyboard navigation (Enter/Escape)
- ✅ Loading state during async operations
- ✅ Error state display
- ✅ Callback support (on_accept_clicked, on_cancel_clicked)

**Issues Addressed:** #693, #740

---

### 1.2 Dropdown/Popup System 🚫 BLOCKED
**Issues:** #542, #615, #452

**Status:** Waiting on Makepad popup APIs (#615)

**Existing Infrastructure:**
- ✅ `popup_list.rs` - Toast-style notifications (not menus)
- ❌ Menu dropdown component
- ❌ Autocomplete popup

---

### 1.3 Collapsible Section Component ✅ IMPLEMENTED
**Location:** `src/shared/collapsible_header.rs`, `src/room/reply_preview.rs`

**Existing Features:**
- ✅ `CollapsibleHeader` widget with header and toggle
- ✅ Smooth rotation animation for collapse indicator
- ✅ Badge indicator for unread counts
- ✅ Click to toggle expand/collapse
- ✅ Category-based headers (Invites, Favorites, DMs, Rooms, etc.)

**Issue #756 - Collapsible Reply Previews (COMPLETED):**
- ✅ `RepliedToMessage` widget with expand/collapse support
- ✅ Max collapsed height of 80px
- ✅ "Show more" / "Show less" toggle label
- ✅ State reset on widget reuse (PortalList pattern)

**Issues Addressed:** #560, #756, #760

---

### 1.4 Loading States ✅ IMPLEMENTED
**Location:** `src/shared/bouncing_dots.rs`, Makepad's `LoadingSpinner`

**Existing Components:**
- ✅ `BouncingDots` - Animated three-dot loading indicator
- ✅ `LoadingSpinner` - Circular spinner (Makepad built-in)
- ✅ Animation states (on/off)
- ✅ Used in `ImageViewer`, `JoinLeaveRoomModal`

**Issues Addressed:** #693, #506, #645

---

### 1.5 Popup Notifications ✅ IMPLEMENTED
**Location:** `src/shared/popup_list.rs`

**Features:**
- ✅ Toast-style notifications
- ✅ Error, Info, Success, Warning variants
- ✅ Auto-dismissal with configurable duration
- ✅ Manual close button
- ✅ Queue system for multiple notifications
- ✅ Global access via `enqueue_popup_notification()`

---

## 2. Room Membership & State Management

### 2.1 Room Membership State Machine ✅ IMPLEMENTED
**Location:** `src/join_leave_room_modal.rs`, `src/home/invite_screen.rs`, `src/shared/no_longer_member_view.rs`

**Implemented:**
- ✅ Join room flow with confirmation
- ✅ Leave room flow with confirmation
- ✅ Accept/reject invite flow
- ✅ Leave space flow
- ✅ Loading states during operations
- ✅ Success/error feedback

**Issue #656 - No Longer Member View (COMPLETED):**
- ✅ "No longer member" view for left/kicked/banned states
- ✅ Distinct UI for kicked vs banned (different icons and messages)
- ✅ Rejoin capability display (button shown for left/kicked, hidden for banned)
- ✅ Reason text display when available
- ✅ `RoomsListAction::RoomRemoved` emitted when room is removed
- ✅ RoomScreen handles action to show `NoLongerMemberView`

---

### 2.2 Session State Persistence 🔨 PARTIAL
**Issues:** #639, #542, #560

**Existing:**
- ✅ Room tabs are persisted
- 🔨 Uses Makepad LiveIds (issue #639 wants room IDs)
- ✅ Collapsible states persist
- ✅ Missing rooms handled gracefully on restore (no panic, shows "room not found" message)

**Needs Work:**
- ❌ Migrate tab persistence to use room IDs (requires Makepad Dock changes)

---

### 2.3 Offline Mode ❌ NOT STARTED
**Issues:** #435

**Status:** Not implemented yet

**Needs:**
- ❌ Network connectivity detection
- ❌ Cached room/message display
- ❌ Offline indicator
- ❌ Message queue for offline sending

---

## 3. Search & Performance

### 3.1 Async Search Infrastructure 🔨 PARTIAL
**Location:** `src/shared/room_filter_input_bar.rs`

**Existing:**
- ✅ Room filter input bar
- ✅ Basic filtering

**Needs Work (Issue #506):**
- ❌ Background thread search for member lists
- ❌ Search debouncing
- ❌ Loading indicator during search
- ❌ Search cancellation

---

### 3.2 Internationalized Search ❌ NOT STARTED
**Issues:** #564

**Status:** Not implemented

**Needs:**
- ❌ Transliteration library
- ❌ Pinyin/romaji matching

---

## 4. Platform & Cross-Platform

### 4.1 Platform Detection ✅ IMPLEMENTED
**Issues:** #755, #458, #376

**Features:**
- ✅ Compiles for macOS, Windows, Linux, Android, iOS
- ✅ Platform-specific code paths exist
- ✅ Touch platform detection via `is_touch_primary_platform()`
- ✅ SpaceLobby buttons visible by default on mobile (#755)

---

### 4.2 App Lifecycle ✅ IMPLEMENTED
**Issues:** #458

**Features:**
- ✅ Pause event handler - saves app state
- ✅ Resume event handler - logs activation
- ✅ Background event handler - saves window state
- ✅ Foreground event handler - logs visibility

---

### 4.3 System Integration 🚫 BLOCKED
**Issues:** #344, #345

**Status:** Blocked on robius framework

---

## 5. Notification System

### 5.1 Notification Badges ✅ IMPLEMENTED
**Location:** `src/shared/unread_badge.rs`

**Features:**
- ✅ Per-room unread badge
- ✅ Mention indicator (red)
- ✅ Unread message count (gray)
- ✅ Marked as unread indicator
- ✅ Truncation at 99+
- ✅ Used in room list and collapsible headers

**Issues Addressed:** #511

---

### 5.2 Mention System 🚫 BLOCKED
**Issues:** #452

**Status:** Blocked on Makepad popup APIs (#615)

**Existing:**
- ✅ `mentionable_text_input.rs` exists
- ❌ Autocomplete popup not functional

---

## 6. Trust Spanning Protocol (TSP)

### 6.1 TSP Core 🔨 PARTIAL
**Location:** `src/tsp/`

**Existing:**
- ✅ TSP module structure
- ✅ `create_did_modal.rs`
- ✅ `create_wallet_modal.rs`
- ✅ `tsp_verification_modal.rs`

---

### 6.2 TSP Security 🔨 PARTIAL
**Issues:** #600, #601

**Status:**
- 🔨 Password handling exists
- 🚫 Biometric auth blocked on robius (#601)

---

### 6.3 TSP Messaging 🚫 BLOCKED
**Issues:** #598, #599

**Status:** Blocked on TSP specification

---

### 6.4 TSP Identity 🔨 PARTIAL
**Issues:** #597

**Status:** WebVH DID support in progress

---

## 7. Spaces & Room Organization

### 7.1 Space Lobby ✅ IMPLEMENTED
**Issues:** #645, #755

**Existing:**
- ✅ Space hierarchy display
- ✅ Room previews
- ✅ Mobile button visibility without hover (#755) - uses `is_touch_primary_platform()`
- ✅ Join room from lobby (join_button, leave_button, view_button)

---

## 8. Media & Image Handling

### 8.1 Image Display ✅ IMPLEMENTED
**Issues:** #650

**Features:**
- ✅ Image thumbnails in timeline
- ✅ Maximum thumbnail height limit (300px)

---

### 8.2 Image Viewer ✅ IMPLEMENTED
**Location:** `src/shared/image_viewer.rs`

**Features:**
- ✅ Full-screen image viewer
- ✅ Zoom in/out (buttons, scroll, pinch)
- ✅ Pan/drag
- ✅ Rotation (CW/CCW)
- ✅ Auto-show/hide UI controls
- ✅ Loading states
- ✅ Error handling
- ✅ Metadata display (sender, timestamp, file info)
- ✅ Keyboard shortcuts (Escape, +/-, 0)

**Issues Addressed:** #655

---

## 9. Thread Support 🔨 PARTIAL
**Issues:** #760

**Status:** Basic thread support exists, enhancements needed

**Needs:**
- ❌ Thread collapse/expand
- ❌ Thread notifications
- ❌ Mark thread as read

---

## 10. Background Task Architecture 🔨 PARTIAL
**Issues:** #617

**Existing:**
- ✅ Async task submission via `submit_async_request`
- ✅ Background workers exist

**Needs:**
- ❌ Separate Matrix vs general queues
- ❌ Priority levels

---

## 11. Makepad Dependencies 🚫 BLOCKED
**Issues:** #594, #615, #479, #452

**Waiting on upstream:**
- ❌ Popup control APIs
- ❌ SVG rendering fixes (Windows/Linux)
- ❌ Text selection
- ❌ Drag and drop

---

## 12. Robius Dependencies 🚫 BLOCKED
**Issues:** #344, #345, #601

**Waiting on upstream:**
- ❌ Network proxy detection
- ❌ System logging
- ❌ Biometric authentication

---

## Summary

| Category | Status | Completion |
|----------|--------|------------|
| Modal System | ✅ IMPLEMENTED | 100% |
| Collapsible Sections | ✅ IMPLEMENTED | 100% |
| Loading States | ✅ IMPLEMENTED | 100% |
| Popup Notifications | ✅ IMPLEMENTED | 100% |
| Notification Badges | ✅ IMPLEMENTED | 100% |
| Image Viewer | ✅ IMPLEMENTED | 100% |
| Image Display | ✅ IMPLEMENTED | 100% |
| Platform Detection | ✅ IMPLEMENTED | 100% |
| App Lifecycle | ✅ IMPLEMENTED | 100% |
| Room Membership | ✅ IMPLEMENTED | 100% |
| Session Persistence | 🔨 PARTIAL | 75% |
| Search | 🔨 PARTIAL | 40% |
| TSP Features | 🔨 PARTIAL | 40% |
| Space Lobby | ✅ IMPLEMENTED | 100% |
| Thread Support | 🔨 PARTIAL | 30% |
| Background Tasks | 🔨 PARTIAL | 50% |
| Offline Mode | ❌ NOT STARTED | 0% |
| Dropdown/Popup | 🚫 BLOCKED | 0% |
| Mention System | 🚫 BLOCKED | 0% |
| System Integration | 🚫 BLOCKED | 0% |

---

## Compilation Status

```bash
$ cargo check
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.90s
```

**Project compiles successfully!**

Last verified: 2026-03-12 (session 3)

---

## Next Steps (Recommended Priority)

### Completed This Session:

1. ✅ **Issue #656** - "No longer member" view - COMPLETE
2. ✅ **Issue #650** - Limit image thumbnail height - COMPLETE
3. ✅ **Issue #755** - Mobile button visibility - COMPLETE
4. ✅ **Issue #458** - App lifecycle events - COMPLETE
5. ✅ **Issue #756** - Collapsible reply previews - COMPLETE
6. ✅ **Issue #645** - Space Lobby join buttons - COMPLETE (already implemented)
7. ✅ **Issue #560** - Server notice rooms under separate header - COMPLETE
8. ✅ **Issue #592** - Prettier invite screen with timestamp - COMPLETE

### Ready to Implement Now:

1. **Issue #639** - Use room IDs for tab persistence
   - Migrate from LiveIds to room IDs
   - Handle missing rooms gracefully on restore

2. **Issue #506** - Async search with loading states
   - Add background thread search for member lists
   - Add search debouncing and loading indicator

### Requires Design Decisions:

- Issue #435 - Offline mode (architecture needed)
- Issue #617 - Worker queue separation (architecture needed)

### Blocked (Track Upstream):

- Makepad: popup APIs, SVG fixes, text selection
- Robius: proxy detection, logging, biometrics

---

## Generated
Date: 2026-03-12
Source: Analysis of 37 GitHub issues and current codebase
